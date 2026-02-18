use acir::{FieldElement, native_types::WitnessMap};
use acir_checker::{BackendType, check_execution};
use nargo::package::Package;
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use nargo_toml::resolve_workspace_from_toml;
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};

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
}
#[test]
fn test_execute_failure_invalid_comptime_bits_decomposition() {
    run_execution_test_failure("test_programs/execution_failure/invalid_comptime_bits_decomposition/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8994() {
    run_execution_test_failure("test_programs/execution_failure/regression_8994/Nargo.toml");
}
#[test]
fn test_execute_failure_slice_insert_failure() {
    run_execution_test_failure("test_programs/execution_failure/slice_insert_failure/Nargo.toml");
}
#[test]
fn test_execute_failure_lambda_from_empty_array_dyn_index() {
    run_execution_test_failure("test_programs/execution_failure/lambda_from_empty_array_dyn_index/Nargo.toml");
}
#[test]
fn test_execute_failure_fold_dyn_index_fail() {
    run_execution_test_failure("test_programs/execution_failure/fold_dyn_index_fail/Nargo.toml");
}
#[test]
fn test_execute_failure_slice_remove_failure() {
    run_execution_test_failure("test_programs/execution_failure/slice_remove_failure/Nargo.toml");
}
#[test]
fn test_execute_failure_div_by_zero_numerator_witness() {
    run_execution_test_failure("test_programs/execution_failure/div_by_zero_numerator_witness/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8231() {
    run_execution_test_failure("test_programs/execution_failure/regression_8231/Nargo.toml");
}
#[test]
fn test_execute_failure_dynamic_index_failure() {
    run_execution_test_failure("test_programs/execution_failure/dynamic_index_failure/Nargo.toml");
}
#[test]
fn test_execute_failure_slice_access_failure() {
    run_execution_test_failure("test_programs/execution_failure/slice_access_failure/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_array_set_known_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_array_set_known_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_empty_composite_array_get() {
    run_execution_test_failure("test_programs/execution_failure/empty_composite_array_get/Nargo.toml");
}
#[test]
fn test_execute_failure_div_by_zero_modulo() {
    run_execution_test_failure("test_programs/execution_failure/div_by_zero_modulo/Nargo.toml");
}
#[test]
fn test_execute_failure_div_by_zero_witness() {
    run_execution_test_failure("test_programs/execution_failure/div_by_zero_witness/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8230() {
    run_execution_test_failure("test_programs/execution_failure/regression_8230/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_5202() {
    run_execution_test_failure("test_programs/execution_failure/regression_5202/Nargo.toml");
}
#[test]
fn test_execute_failure_u128_multiplication_overflow() {
    run_execution_test_failure("test_programs/execution_failure/u128_multiplication_overflow/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_7759() {
    run_execution_test_failure("test_programs/execution_failure/regression_7759/Nargo.toml");
}
#[test]
fn test_execute_failure_invalid_comptime_bytes_decomposition() {
    run_execution_test_failure("test_programs/execution_failure/invalid_comptime_bytes_decomposition/Nargo.toml");
}
#[test]
fn test_execute_failure_option_expect() {
    run_execution_test_failure("test_programs/execution_failure/option_expect/Nargo.toml");
}
#[test]
fn test_execute_failure_assert_msg_runtime() {
    run_execution_test_failure("test_programs/execution_failure/assert_msg_runtime/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8195() {
    run_execution_test_failure("test_programs/execution_failure/regression_8195/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_7128() {
    run_execution_test_failure("test_programs/execution_failure/regression_7128/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_slice_get_known_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_slice_get_known_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_mocks_in_execution() {
    run_execution_test_failure("test_programs/execution_failure/mocks_in_execution/Nargo.toml");
}
#[test]
fn test_execute_failure_dyn_index_fail_nested_array() {
    run_execution_test_failure("test_programs/execution_failure/dyn_index_fail_nested_array/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8229() {
    run_execution_test_failure("test_programs/execution_failure/regression_8229/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_slice_get_unknown_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_slice_get_unknown_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_array_get_known_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_array_get_known_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_div_by_zero_constants() {
    run_execution_test_failure("test_programs/execution_failure/div_by_zero_constants/Nargo.toml");
}
#[test]
fn test_execute_failure_hashmap_load_factor() {
    run_execution_test_failure("test_programs/execution_failure/hashmap_load_factor/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_array_get_unknown_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_array_get_unknown_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_brillig_assert_fail() {
    run_execution_test_failure("test_programs/execution_failure/brillig_assert_fail/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8233() {
    run_execution_test_failure("test_programs/execution_failure/regression_8233/Nargo.toml");
}
#[test]
fn test_execute_failure_unused_array_set_unknown_index_out_of_bounds() {
    run_execution_test_failure("test_programs/execution_failure/unused_array_set_unknown_index_out_of_bounds/Nargo.toml");
}
#[test]
fn test_execute_failure_lambda_from_empty_array() {
    run_execution_test_failure("test_programs/execution_failure/lambda_from_empty_array/Nargo.toml");
}
#[test]
fn test_execute_failure_fold_nested_brillig_assert_fail() {
    run_execution_test_failure("test_programs/execution_failure/fold_nested_brillig_assert_fail/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_9266() {
    run_execution_test_failure("test_programs/execution_failure/regression_9266/Nargo.toml");
}
#[test]
fn test_execute_failure_regression_8175() {
    run_execution_test_failure("test_programs/execution_failure/regression_8175/Nargo.toml");
}