//! Helper function for tests

use std::path::Path;

use acir_checker::{
    BackendType, 
    Error, 
    Output, 
    check_program
};

use noirc_driver::{
    CompileOptions, 
    CompiledProgram, 
    file_manager_with_stdlib, 
    prepare_crate
};

use noirc_frontend::{
    hir::{
        Context, 
        def_map::parse_file
    }, 
    node_interner::FuncId
};

#[allow(unused)]
macro_rules! add_decl {
    ($program : literal) => (
        concat!($program, 
            "unconstrained fn verify_assert(exp: bool){keep()}
            unconstrained fn verify_assume(exp: bool){keep()}
            #[oracle(keep)]
            unconstrained fn keep(){}"
        )
    )
}

#[allow(unused)]
pub(crate) fn is_verified(
    source: &str,
    backend: BackendType,
    strict: bool
) -> bool {
    compile_and_check(source, backend, strict)
        .unwrap()
        .into_iter()
        .all(|(output, _)| output.is_verified())
}

#[allow(unused)]
pub(crate) fn is_falsified(
    source: &str,
    backend: BackendType,
    strict: bool
) -> bool {
    compile_and_check(source, backend, strict)
        .unwrap()
        .into_iter()
        .all(|(output, _)| output.is_falsified())
}

#[allow(unused)]
pub(crate) fn is_err(
    source: &str,
    backend: BackendType,
    strict: bool
) -> bool {
    compile_and_check(source, backend, strict).is_err()
}

#[allow(unused)]
pub(crate) fn no_ver_asserts(
    source: &str,
    backend: BackendType,
    strict: bool
) -> bool {
    compile_and_check(source, backend, strict).unwrap().is_empty()
}

pub(crate) fn compile_and_check(
    source: &str,
    backend: BackendType,
    strict: bool,
) -> Result<Vec<(Output, usize)>, Error> {
    let comp_programs = compile_noir_source_from_string(source);

    let mut result = Vec::new();

    for program in comp_programs {
        let circuit = program.program.functions.first().unwrap();
        result.extend(check_program(circuit, program.brillig_names, backend, strict)?);
    }
    Ok(result)
}

fn compile_noir_source_from_string(prog_str: &str) -> Vec<CompiledProgram> {
    let root = Path::new("");
    let file_name = Path::new("main.nr");
    let mut file_manager = file_manager_with_stdlib(root);
    file_manager.add_file_with_source(
        file_name, 
        prog_str.to_owned()
    ).expect("adding file to manager should not fail when file manager is empty");

    let parsed_files = file_manager
        .as_file_map()
        .all_file_ids()
        .map(|&file_id| (file_id, parse_file(&file_manager, file_id)))
        .collect();

    let mut context = Context::new(file_manager, parsed_files);
    let root_crate_id = prepare_crate(&mut context, file_name);

    let ((), _) =
        noirc_driver::check_crate(&mut context, root_crate_id, &Default::default()).unwrap();
    let options = CompileOptions::default();

    let main_id = context.get_main_function(&root_crate_id).unwrap();

    let mut progs = Vec::new();
    let main_prog = noirc_driver::compile_no_check(
        &mut context, 
        &options, 
        main_id, 
        None, 
        false
    ).unwrap();
    progs.push(main_prog);

    let formal_functions: Vec<(String, FuncId)> = context
        .get_all_formal_functions_in_crate(&root_crate_id)
        .into_iter()
        .collect();

    for (_name, func_id) in formal_functions {
        if func_id == main_id {
            continue
        }
        let program = noirc_driver::compile_no_check(
            &mut context,
            &options, 
            func_id, 
            None, 
            false
        ).unwrap();
        progs.push(program);
    }
    progs
}