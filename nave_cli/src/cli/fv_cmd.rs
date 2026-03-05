//! Nargo CLI formal-verify command
//! More details in: tooling/acir_checker

use clap::{
    Args, 
    ValueEnum
};
use std::io::{
    self,
    Write,
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use acir_checker::{
    BackendType, 
    Output, 
    check_program,
    check_execution as check_execution_func,
};
use acvm::acir::circuit::AcirOpcodeLocation;
use nargo::{
    package::Package,
    workspace::Workspace,
    insert_all_files_for_workspace_into_file_manager, 
    parse_all, 
    prepare_package
};
use nargo_toml::PackageSelection;
use noir_artifact_cli::{
    Artifact,
    fs::inputs::read_inputs_from_file,
};
use noirc_driver::{
    CompileOptions, 
    check_crate, 
    compile_no_check,
};

use noirc_artifacts::program::CompiledProgram;

use noirc_errors::{
    Location, 
    Span,
};
use noirc_frontend::node_interner::FuncId;

use super::{
    LockType, 
    PackageOptions, 
    WorkspaceCommand
};
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use crate::errors::CliError;

/// Formally verify the functions of a compiled program using an SMT solver.
#[derive(Debug, Clone, Args)]
// #[clap(visible_alias = "v")]
pub(crate) struct FormalVerifyCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    #[clap(flatten)]
    formal_verify_options: FormalVerifyOptions,
}

#[derive(Args, Clone, Debug, Default)]
struct FormalVerifyOptions {
    #[clap(long, value_enum, default_value = "ff-gb")]
    pub backend: BackendOption,

    #[clap(long, default_value = "false")]
    pub relaxed: bool,

    #[clap(long, default_value = "false")]
    pub check_execution: bool,

    #[clap(long, default_value = "false")]
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct BackendOption(BackendType);

impl ValueEnum for BackendOption {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            BackendOption(BackendType::FfGb),
            BackendOption(BackendType::FfSplit),
            BackendOption(BackendType::Int),
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self.0 {
            BackendType::FfGb => Some(
                clap::builder::PossibleValue::new("ff-gb")
                    .help("Finite field backend using Groebner basis solver"),
            ),
            BackendType::FfSplit => Some(
                clap::builder::PossibleValue::new("ff-split")
                    .help("Finite field backend using split solver"),
            ),
            BackendType::Int => {
                Some(clap::builder::PossibleValue::new("int").help("Integer backend"))
            }
        }
    }
}

impl WorkspaceCommand for FormalVerifyCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Compiles artifacts.
        LockType::Exclusive
    }
}

pub(crate) fn run(
    args: FormalVerifyCommand,
    workspace: Workspace
) -> Result<(), CliError> {
    // Compile the full workspace in order to generate any build artifacts.
    let debug_compile_stdin = None;
    compile_workspace_full(&workspace, &args.compile_options, debug_compile_stdin).map_err(|e| {
        CliError::Generic(format!("Failed to compile workspace: {}", e))
    })?;

    let backend = args.formal_verify_options.backend.0;
    let strict = !args.formal_verify_options.relaxed;
    let check_execution = args.formal_verify_options.check_execution;
    
    // Go over binary packages that have been created to verify their functions.
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let artifact = Artifact::read_from_file(&program_artifact_path)?;

        let compiled_program: CompiledProgram = match artifact {
            Artifact::Program(program) => program.into(),
            Artifact::Contract(_) => {
                return Err(CliError::Generic("Only works for Programs.".to_string()));
            }
        };

        if check_execution {
            let prover_path = package.root_dir.join("Prover.toml");
            let (input_map, opt_output) = read_inputs_from_file(&prover_path, &compiled_program.abi)?;
            let witness_map = compiled_program.abi.encode(&input_map, opt_output)?;
            let circuit = compiled_program.program.functions.first().unwrap();
            let output = check_execution_func(
                witness_map,
                circuit,
                backend,
                strict,
                &vec![]
            ).map_err(|e| {
                CliError::Generic(format!(
                    "Failed to check execution for package {}: {}",
                    package.name, e
                ))
            })?;
            display_exec_result(
                &package.name.to_string(),
                &output, 
            ).map_err(|e| {
                CliError::Generic(format!(
                    "Failed to display execution results {}: {}",
                    package.name, e)
                )
            })?;
            return Ok(());
        }

        println!(
            "Formally verifying using backend {:?} with strict mode set to {}.",
            args.formal_verify_options.backend.0, !args.formal_verify_options.relaxed
        );

        // Run formal verifier for the compiled program which has main funtion
        // as the entry point (are there any scenarios in which it won't be the
        // main function?)
        run_formal_harness(&args, package, &compiled_program, "main")?;
    }

    // Collect all functions with formal attribute and run formal verifier
    //
    // TODO: Consider restrictions on using formally annotated functions in production.
    // Calling a test-like or verification function from main is technically allowed
    // in Rust, but goes against common #[cfg(test)] conventions. Since we already
    // avoid re-analyzing main, we could extend this to skip verifying any functions
    // reachable from main.
    //
    // TODO: Consider using full workspace compilation, and the same file manager
    // it uses instead of initializing those again in the following section.
    // Refactoring might be required after that.
    
    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut file_manager);
    let parsed_files = parse_all(&file_manager);

    let options = &args.compile_options;
    for package in workspace.into_iter() {
        let (mut context, crate_id) = prepare_package(&file_manager, &parsed_files, package);
        check_crate(&mut context, crate_id, &options).map_err(|e|   
            CliError::Generic(format!(
            "Failed to check crate for {:?}: {:?}",
            crate_id, e
        )))?;

        let formal_functions: Vec<(String, FuncId)> = context
            .get_all_formal_functions_in_crate(&crate_id)
            .into_iter()
            .collect();

        let opt_main_func_id = context.get_main_function(&crate_id);
        for (func_name, func_id) in formal_functions {
            if Some(func_id) == opt_main_func_id {
                continue
            }
            let program = compile_no_check(
                &mut context,
                &options, 
                func_id, 
                None, 
                false
            ).map_err(|e|   
                CliError::Generic(format!(
                "Failed to compile function {:?}: {:?}",
                func_name, e
            )))?;
            run_formal_harness(&args, package, &program, &func_name)?;
        }
    }
    Ok(())
}

// Private ------------------------------------------------

fn run_formal_harness(
    args: &FormalVerifyCommand,
    package: &Package,
    compiled_program: &CompiledProgram,
    func_name: &str,
) -> Result<(), CliError> {
    let backend = args.formal_verify_options.backend.0;
    let strict = !args.formal_verify_options.relaxed;
    let is_verbose = args.formal_verify_options.verbose;

    let circuit = compiled_program.program.functions.first().unwrap();
    let brillig_names = compiled_program.program.unconstrained_functions.iter().map(|f| f.function_name.clone()).collect();
    let outputs = check_program(circuit, brillig_names, backend, strict).map_err(|e| {
        CliError::Generic(format!(
            "Failed to check program for package {}: {}",
            package.name, e
        ))
    })?;

    if !compiled_program.debug.is_empty() {
        display_verify_result(
            &compiled_program, 
            &package.name.to_string(), 
            func_name,
            &outputs,
            is_verbose
        ).map_err(|e| {
            CliError::Generic(format!(
                "Failed to display verification results {}: {}",
                package.name, e)
            )
        })?;
    }
    Ok(())
}

// Get verification condition from the source program given `Location` of the
// condition from debug information
fn get_cond_from(location: Location, program: &CompiledProgram) -> Option<String> {
    let span = location.span;
    let file_id = location.file;
    let debug_file = program.file_map.get(&file_id)?;
    let prog_source = &debug_file.source;
    let (line, column) = line_and_column_from(&span, prog_source,);
    extract_expr(line, column, prog_source)
}

// Extract verification condition from the source program given line number and 
// column
fn extract_expr(line_num: u32, col: u32, source: &str) -> Option<String> {
    let verify_cond_line = source
        .lines() // iterator over lines
        .nth((line_num - 1) as usize) // adjust for 0-indexing
        .map(|s| s.to_string())?;

    let stmt_substr = &verify_cond_line[(col as usize)..];

    // Extracting the condition string from assert statement
    let cond_start = stmt_substr.find('(')?;
    let mut cond_end = stmt_substr.rfind(')')?;
    let stmt_end = stmt_substr.find(';')?;
    if stmt_end < cond_end {
        cond_end = stmt_end - 1;
    }
    (cond_start < cond_end).then(|| stmt_substr[cond_start + 1..cond_end].to_string())
}

// Utility function to get line number and column from `Span`
fn line_and_column_from(span: &Span, source: &str) -> (u32, u32) {
    let mut line = 1;
    let mut column = 0;

    for (i, char) in source.chars().enumerate() {
        column += 1;

        if char == '\n' {
            line += 1;
            column = 0;
        }

        if span.start() <= i as u32 {
            break;
        }
    }
    (line, column)
}

// Pretty print result of executing program with given inputs, and expected outputs (if any).
fn display_exec_result(
    package_name: &str,
    output: &Output,
) -> Result<(), io::Error> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    write!(writer, "\n[")?;
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;

    write!(writer, "{}", package_name)?;
    writer.reset()?;
    write!(writer, "] Execution check for package")?;
    writer.reset()?;
    writer.flush()?;

    match output {
        Output::Falsified(_) => {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            writeln!(writer, "Checked")?;
        }
        Output::Verified => {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
            writeln!(writer, "Failed")?;
        }
        Output::Unknown => {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            writeln!(writer, "Unknown")?;
        }
    }
    writer.reset()?;
    writer.flush()?;

    Ok(())
}

// Pretty print result of running formal verifier on program ACIR
// TODO: Need more tidying up.
fn display_verify_result(
    compiled_program: &CompiledProgram,
    package_name: &str,
    func_name: &str,
    fv_result: &Vec<(Output, usize)>,
    is_verbose: bool,
) -> Result<(), io::Error> {
    let writer = StandardStream::stderr(ColorChoice::Always);
    let mut writer = writer.lock();

    write!(writer, "\n[")?;
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;

    write!(writer, "{}", package_name)?;
    writer.reset()?;
    write!(writer, "] Checked ")?;
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
    write!(writer, "{func_name} \n")?;
    writer.reset()?;
    writer.flush()?;

    if !is_verbose {
        let is_verified = fv_result.iter().all(|(output, _)| output.is_verified());
        if is_verified {
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
            writeln!(writer, "All assertions verified")?;
            writer.reset()?;
            write!(writer, "\n[")?;
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(writer, "{}", package_name)?;
            writer.reset()?;
            writeln!(writer, "]")?;
            writer.flush()?;

            return Ok(())
        }
    }

    let acir_locations = &compiled_program.debug[0].acir_locations;
    let location_tree = &compiled_program.debug[0].location_tree;

    for (output, call_loc) in fv_result {
        // Placeholder value for the verification condition
        let mut condition = "<-->".to_string();

        // Tries to get source program location (and then verification condition)
        // using opcode location of the brillig call (`call_loc`) in the circuit 
        match acir_locations.get(&AcirOpcodeLocation::new(*call_loc)) {
            Some(call_stack_id) => {
                let locations = location_tree.get_call_stack(*call_stack_id);
                if !locations.is_empty() {
                    if let Some(c) = get_cond_from(locations[0], &compiled_program) {
                        condition = c;
                    }
                }
            },
            None => {}
        };

        if !is_verbose {
            if output.is_falsified() {
                write!(writer, "Assert ")?;
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                write!(writer, "\"{}\" : ", condition)?;
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                writeln!(writer, "Falsified")?;
                writer.reset()?;
            }
        } else {
            write!(writer, "Assert ")?;
            writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(writer, "\"{}\" : ", condition)?;
            
            match output {
                Output::Falsified(model) => {
                    writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                    writeln!(writer, "Falsified")?;
                    writer.reset()?;
                    writeln!(writer, "Model---\n[\"{:?}\"]", model)?;
                    
                    // Print input parameters and values (if any)
                    let param_names = compiled_program.abi.parameter_names();

                    if !param_names.is_empty() {
                        let witness_ids_iter = compiled_program.program.functions[0].private_parameters.iter();
                        writeln!(writer, "\nFunction parameters (name, witness_id, and value)---")?;
                        for (name, id) in param_names.iter().zip(witness_ids_iter) {
                            let id_str = id.to_string();
                            let val = model.get(&id_str);
                            writeln!(writer, "{name}: {} -> {:?}", id_str, val)?;
                        }
                    }
                    
                    // Print return values (if any)
                    // functions[0] in the following correspond to the main function which must be present
                    // in the main source file
                    let return_vals = &compiled_program.program.functions[0].return_values.0;

                    if !return_vals.is_empty() {
                        writeln!(writer, "\nReturn values (witness_id, and value)---")?;
                        for id in return_vals {
                            let id_str = id.to_string();
                            let val = model.get(&id_str);
                            writeln!(writer, "{} -> {:?}", id_str, val)?;
                        }
                    }
                }
                Output::Verified => {
                    writer.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                    writeln!(writer, "Verified")?;
                }
                Output::Unknown => {
                    writer.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                    writeln!(writer, "Unknown")?;
                }
            }
            writer.reset()?;
        }
    }

    write!(writer, "\n[")?;
    writer.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
    write!(writer, "{}", package_name)?;
    writer.reset()?;
    writeln!(writer, "]")?;
    writer.flush()?;

    Ok(())
}
