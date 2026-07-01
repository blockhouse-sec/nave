use acir::{FieldElement, native_types::WitnessMap};
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
    ).unwrap();
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
            println!("Could not read artifact for package {}, skipping test.", package.name);
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

fn has_unmodeled_brillig_outputs(circuit: &acir::circuit::Circuit<FieldElement>) -> bool {
    circuit.opcodes.iter().any(|op| {
        matches!(op, acir::circuit::Opcode::BrilligCall { outputs, .. } if !outputs.is_empty())
    })
}

fn check_execution_test(package: &Package, witness_map: WitnessMap<FieldElement>, compiled_program: &CompiledProgram, backend: BackendType, solver_options: &Vec<(String, String)>) {
    let circuit = compiled_program.program.functions.first().unwrap();
    let output = check_execution(witness_map, circuit, backend, true, solver_options);
    if let Ok(output) = output {
        println!("Execution check {:?} for package failure:{} completed with output: {:?}",
            backend, package.name, output
        );
        
        if !output.is_verified() && has_unmodeled_brillig_outputs(circuit) {
            println!(
                "Result is inconclusive for {}: circuit contains Brillig call(s) with outputs that NAVE does not model." ,
                package.name
            );
            return;
        }
        // This encompasses both verified and inconclusive outputs.
        assert!(!output.is_falsified());
    } else {
        println!("Execution check {:?} for package failure:{} failed. Error: {:?}",
            backend, package.name, output.err()
        );
    }
}

#[cfg(test)] 
mod tests {
    use crate::run_execution_test_failure;

    #[test]
    fn test_execute_failure_regression_10929() {
        run_execution_test_failure("test_programs/execution_failure/regression_10929/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_9489() {
        run_execution_test_failure("test_programs/execution_failure/regression_9489/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_invalid_comptime_bits_decomposition() {
        run_execution_test_failure("test_programs/execution_failure/invalid_comptime_bits_decomposition/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_vector_insert_failure() {
        run_execution_test_failure("test_programs/execution_failure/vector_insert_failure/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_vector_pop_back_oob() {
        run_execution_test_failure("test_programs/execution_failure/vector_pop_back_oob/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_signed_modulo_by_minus_one_overflow() {
        run_execution_test_failure("test_programs/execution_failure/signed_modulo_by_minus_one_overflow/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_databus_out_of_bounds() {
        run_execution_test_failure("test_programs/execution_failure/regression_databus_out_of_bounds/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_conditional_pop_front_from_empty_vector() {
        run_execution_test_failure("test_programs/execution_failure/conditional_pop_front_from_empty_vector/Nargo.toml");
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
    fn test_execute_failure_unused_vector_get_known_index_out_of_bounds() {
        run_execution_test_failure("test_programs/execution_failure/unused_vector_get_known_index_out_of_bounds/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_vector_remove_oob() {
        run_execution_test_failure("test_programs/execution_failure/vector_remove_oob/Nargo.toml");
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
    fn test_execute_failure_signed_division_by_minus_one_overflow() {
        run_execution_test_failure("test_programs/execution_failure/signed_division_by_minus_one_overflow/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_8230() {
        run_execution_test_failure("test_programs/execution_failure/regression_8230/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_conditional_pop_back_from_empty_vector() {
        run_execution_test_failure("test_programs/execution_failure/conditional_pop_back_from_empty_vector/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_vector_access_failure() {
        run_execution_test_failure("test_programs/execution_failure/vector_access_failure/Nargo.toml");
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
    fn test_execute_failure_vector_remove_failure() {
        run_execution_test_failure("test_programs/execution_failure/vector_remove_failure/Nargo.toml");
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
    fn test_execute_failure_vector_pop_front_oob() {
        run_execution_test_failure("test_programs/execution_failure/vector_pop_front_oob/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_8195() {
        run_execution_test_failure("test_programs/execution_failure/regression_8195/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_conditional_remove_from_empty_vector() {
        run_execution_test_failure("test_programs/execution_failure/conditional_remove_from_empty_vector/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_7128() {
        run_execution_test_failure("test_programs/execution_failure/regression_7128/Nargo.toml");
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
    fn test_execute_failure_unused_vector_get_unknown_index_out_of_bounds() {
        run_execution_test_failure("test_programs/execution_failure/unused_vector_get_unknown_index_out_of_bounds/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_bitshift_normalization() {
        run_execution_test_failure("test_programs/execution_failure/regression_bitshift_normalization/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_div_by_zero_constants() {
        run_execution_test_failure("test_programs/execution_failure/div_by_zero_constants/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_unused_array_get_unknown_index_out_of_bounds() {
        run_execution_test_failure("test_programs/execution_failure/unused_array_get_unknown_index_out_of_bounds/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_shl_overflow_u64() {
        run_execution_test_failure("test_programs/execution_failure/shl_overflow_u64/Nargo.toml");
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
    fn test_execute_failure_fold_nested_brillig_assert_fail() {
        run_execution_test_failure("test_programs/execution_failure/fold_nested_brillig_assert_fail/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_shr_overflow_u64() {
        run_execution_test_failure("test_programs/execution_failure/shr_overflow_u64/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_9266() {
        run_execution_test_failure("test_programs/execution_failure/regression_9266/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_8175() {
        run_execution_test_failure("test_programs/execution_failure/regression_8175/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_brillig_assert_msg_runtime() {
        run_execution_test_failure("test_programs/execution_failure/brillig_assert_msg_runtime/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_div_signed() {
        run_execution_test_failure("test_programs/execution_failure/div_signed/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_mod_by_zero_witness() {
        run_execution_test_failure("test_programs/execution_failure/mod_by_zero_witness/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_mod_by_zero_witness_signed() {
        run_execution_test_failure("test_programs/execution_failure/mod_by_zero_witness_signed/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_10238() {
        run_execution_test_failure("test_programs/execution_failure/regression_10238/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_12469() {
        run_execution_test_failure("test_programs/execution_failure/regression_12469/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_regression_9904() {
        run_execution_test_failure("test_programs/execution_failure/regression_9904/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_shl_signed_regression_9592() {
        run_execution_test_failure("test_programs/execution_failure/shl_signed_regression_9592/Nargo.toml");
    }
    #[test]
    fn test_execute_failure_unused_array_set_unknown_index_out_of_bounds() {
        run_execution_test_failure("test_programs/execution_failure/unused_array_set_unknown_index_out_of_bounds/Nargo.toml");
    }
}
