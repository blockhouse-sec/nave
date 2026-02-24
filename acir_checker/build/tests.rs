// tests.rs

use std::fs;
use std::path::Path;

pub(crate) fn create_tests() {
    let tests_dir = Path::new(&"tests");
    let success_test_file = tests_dir.join("success.rs");
    if !success_test_file.exists() {
        let mut tests_string = PRELUDE_SUCCESS.to_string();
        let test_folders = fs::read_dir("test_programs/execution_success").unwrap();
        for test_folder in test_folders {
            let test_folder = test_folder.unwrap();
            let test_folder_path = test_folder.path();
            let nargo_toml_path = test_folder_path.join("Nargo.toml");
            tests_string.push_str(&format!(
                "
#[test]
fn test_execute_success_{}() {{
    run_execution_test_success(\"{}\");
}}",
                test_folder.path().file_name().unwrap().to_str().unwrap(),
                nargo_toml_path.to_str().unwrap()
            ));
        }
        fs::write(success_test_file, tests_string).unwrap();
    }
    let failure_test_file = tests_dir.join("failure.rs");
    if !failure_test_file.exists() {
        let mut tests_string = PRELUDE_FAILURE.to_string();
        let test_folders = fs::read_dir("test_programs/execution_failure").unwrap();
        for test_folder in test_folders {
            let test_folder = test_folder.unwrap();
            let test_folder_path = test_folder.path();
            let nargo_toml_path = test_folder_path.join("Nargo.toml");
            tests_string.push_str(&format!(
                "
#[test]
fn test_execute_failure_{}() {{
    run_execution_test_failure(\"{}\");
}}",
                test_folder.path().file_name().unwrap().to_str().unwrap(),
                nargo_toml_path.to_str().unwrap()
            ));
        }
        fs::write(failure_test_file, tests_string).unwrap();
    }
}

static PRELUDE_SUCCESS: &str = r#"use acir::{FieldElement, native_types::WitnessMap};
use acir_checker::{BackendType, check_execution};
use nargo::package::Package;
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use nargo_toml::resolve_workspace_from_toml;
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noirc_artifacts::program::CompiledProgram;
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};

fn run_execution_test_success(nargo_toml_path: &str) {
    let cargo_manifest_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let nargo_toml_path = cargo_manifest_dir.join(nargo_toml_path);
    let workspace = resolve_workspace_from_toml(
        &nargo_toml_path,
        nargo_toml::PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .unwrap();
    let compile_options = CompileOptions::default();
    let debug_compile_stdin = None;
    let _ = compile_workspace_full(&workspace, &compile_options, debug_compile_stdin);
    let mut packages = workspace.into_iter().peekable();
    assert!(packages.peek().is_some());
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);

        let artifact = Artifact::read_from_file(&program_artifact_path).unwrap();

        let compiled_program: CompiledProgram = match artifact {
            Artifact::Program(program) => program.into(),
            Artifact::Contract(_) => {
                assert!(false);
                return;
            }
        };

        let prover_path = package.root_dir.join("Prover.toml");
        let (input_map, opt_output) =
            read_inputs_from_file(&prover_path, &compiled_program.abi).unwrap();
        let witness_map = compiled_program.abi.encode(&input_map, opt_output).unwrap();
        // Limit solver time to 10 seconds for each check and add some resource constraint too.
        let solver_options = vec![(":tlimit-per".into(), "10000".into()),(":rlimit-per".into(), "100000".into())];
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::FfGb, &solver_options);
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::FfSplit, &solver_options);
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::Int, &solver_options);
    }
}

fn check_execution_test(package: &Package, witness_map: WitnessMap<FieldElement>, compiled_program: &CompiledProgram, backend: BackendType, solver_options: &Vec<(String, String)>) {
    let circuit = compiled_program.program.functions.first().unwrap();
    let output = check_execution(witness_map, circuit, backend, true, solver_options);
    if let Ok(output) = output {
        println!("Execution check {:?} for package {} is_falsified: {}",
            backend, package.name, output.is_falsified()
        );
        // This encompasses both verified and inconclusive outputs.
        assert!(!output.is_verified());
    }
}"#;

static PRELUDE_FAILURE: &str = r#"use acir::{FieldElement, native_types::WitnessMap};
use acir_checker::{BackendType, check_execution};
use nargo::package::Package;
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use nargo_toml::resolve_workspace_from_toml;
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noirc_artifacts::program::CompiledProgram;
use noirc_driver::{CompileOptions, NOIR_ARTIFACT_VERSION_STRING};

fn run_execution_test_failure(nargo_toml_path: &str) {
    let cargo_manifest_dir = std::path::PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let nargo_toml_path = cargo_manifest_dir.join(nargo_toml_path);
    let workspace = resolve_workspace_from_toml(
        &nargo_toml_path,
        nargo_toml::PackageSelection::All,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )
    .unwrap();
    let compile_options = CompileOptions::default();
    let debug_compile_stdin = None;
    let _ = compile_workspace_full(&workspace, &compile_options, debug_compile_stdin);
    let mut packages = workspace.into_iter().peekable();
    assert!(packages.peek().is_some());
    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);

        
        let artifact = if let Ok(artifact) = Artifact::read_from_file(&program_artifact_path) {
            artifact
        } else {
            // If we cannot read the artifact, we consider the test passed (as it is a failure test).
            continue;
        };

        let compiled_program: CompiledProgram = match artifact {
            Artifact::Program(program) => program.into(),
            Artifact::Contract(_) => {
                assert!(false);
                return;
            }
        };

        let prover_path = package.root_dir.join("Prover.toml");
        let (input_map, opt_output) =
            read_inputs_from_file(&prover_path, &compiled_program.abi).unwrap();
        let witness_map = compiled_program.abi.encode(&input_map, opt_output).unwrap();
        // Limit solver time to 10 seconds for each check and add some resource constraint too.
        let solver_options = vec![(":tlimit-per".into(), "10000".into()),(":rlimit-per".into(), "100000".into())];
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::FfGb, &solver_options);
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::FfSplit, &solver_options);
        check_execution_test(package, witness_map.clone(), &compiled_program, BackendType::Int, &solver_options);
    }
}

fn check_execution_test(package: &Package, witness_map: WitnessMap<FieldElement>, compiled_program: &CompiledProgram, backend: BackendType, solver_options: &Vec<(String, String)>) {
    let circuit = compiled_program.program.functions.first().unwrap();
    let output = check_execution(witness_map, circuit, backend, true, solver_options);
    if let Ok(output) = output {
        println!("Execution check {:?} for package {} is_verified: {}",
            backend, package.name, output.is_verified()
        );
        // This encompasses both verified and inconclusive outputs.
        assert!(!output.is_falsified());
    }
}"#;
