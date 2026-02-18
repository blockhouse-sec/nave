use acir::{FieldElement, native_types::WitnessMap};
use acir_checker::{BackendType, check_execution};
use nargo::package::Package;
use nargo_cli::cli::compile_cmd::compile_workspace_full;
use nargo_toml::resolve_workspace_from_toml;
use noir_artifact_cli::{Artifact, fs::inputs::read_inputs_from_file};
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};

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
}
#[test]
fn test_execute_success_lambda_from_array() {
    run_execution_test_success("test_programs/execution_success/lambda_from_array/Nargo.toml");
}
#[test]
fn test_execute_success_fold_basic() {
    run_execution_test_success("test_programs/execution_success/fold_basic/Nargo.toml");
}
#[test]
fn test_execute_success_modulus() {
    run_execution_test_success("test_programs/execution_success/modulus/Nargo.toml");
}
#[test]
fn test_execute_success_pedersen_check() {
    run_execution_test_success("test_programs/execution_success/pedersen_check/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_pedersen() {
    run_execution_test_success("test_programs/execution_success/brillig_pedersen/Nargo.toml");
}
#[test]
fn test_execute_success_generics() {
    run_execution_test_success("test_programs/execution_success/generics/Nargo.toml");
}
#[test]
fn test_execute_success_loop_invariant_regression() {
    run_execution_test_success("test_programs/execution_success/loop_invariant_regression/Nargo.toml");
}
#[test]
fn test_execute_success_simple_shift_left_right() {
    run_execution_test_success("test_programs/execution_success/simple_shift_left_right/Nargo.toml");
}
#[test]
fn test_execute_success_ecdsa_secp256r1_3x() {
    run_execution_test_success("test_programs/execution_success/ecdsa_secp256r1_3x/Nargo.toml");
}
#[test]
fn test_execute_success_xor() {
    run_execution_test_success("test_programs/execution_success/xor/Nargo.toml");
}
#[test]
fn test_execute_success_simple_radix() {
    run_execution_test_success("test_programs/execution_success/simple_radix/Nargo.toml");
}
#[test]
fn test_execute_success_as_witness() {
    run_execution_test_success("test_programs/execution_success/as_witness/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7744() {
    run_execution_test_success("test_programs/execution_success/regression_7744/Nargo.toml");
}
#[test]
fn test_execute_success_array_rc_regression_7842() {
    run_execution_test_success("test_programs/execution_success/array_rc_regression_7842/Nargo.toml");
}
#[test]
fn test_execute_success_simple_bitwise() {
    run_execution_test_success("test_programs/execution_success/simple_bitwise/Nargo.toml");
}
#[test]
fn test_execute_success_a_4_sub() {
    run_execution_test_success("test_programs/execution_success/a_4_sub/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8305() {
    run_execution_test_success("test_programs/execution_success/regression_8305/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8761() {
    run_execution_test_success("test_programs/execution_success/regression_8761/Nargo.toml");
}
#[test]
fn test_execute_success_regression_4709() {
    run_execution_test_success("test_programs/execution_success/regression_4709/Nargo.toml");
}
#[test]
fn test_execute_success_traits_in_crates_2() {
    run_execution_test_success("test_programs/execution_success/traits_in_crates_2/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6451() {
    run_execution_test_success("test_programs/execution_success/regression_6451/Nargo.toml");
}
#[test]
fn test_execute_success_mutate_array_copy() {
    run_execution_test_success("test_programs/execution_success/mutate_array_copy/Nargo.toml");
}
#[test]
fn test_execute_success_last_uses_regression_8935() {
    run_execution_test_success("test_programs/execution_success/last_uses_regression_8935/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_in_slice() {
    run_execution_test_success("test_programs/execution_success/nested_array_in_slice/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_rc_regression_6123() {
    run_execution_test_success("test_programs/execution_success/brillig_rc_regression_6123/Nargo.toml");
}
#[test]
fn test_execute_success_strings() {
    run_execution_test_success("test_programs/execution_success/strings/Nargo.toml");
}
#[test]
fn test_execute_success_global_var_regression_simple() {
    run_execution_test_success("test_programs/execution_success/global_var_regression_simple/Nargo.toml");
}
#[test]
fn test_execute_success_simple_shield() {
    run_execution_test_success("test_programs/execution_success/simple_shield/Nargo.toml");
}
#[test]
fn test_execute_success_reference_counts_inliner_max() {
    run_execution_test_success("test_programs/execution_success/reference_counts_inliner_max/Nargo.toml");
}
#[test]
fn test_execute_success_integer_array_indexing() {
    run_execution_test_success("test_programs/execution_success/integer_array_indexing/Nargo.toml");
}
#[test]
fn test_execute_success_dont_deduplicate_call() {
    run_execution_test_success("test_programs/execution_success/dont_deduplicate_call/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8212() {
    run_execution_test_success("test_programs/execution_success/regression_8212/Nargo.toml");
}
#[test]
fn test_execute_success_reference_cancelling() {
    run_execution_test_success("test_programs/execution_success/reference_cancelling/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_with_refs() {
    run_execution_test_success("test_programs/execution_success/nested_array_with_refs/Nargo.toml");
}
#[test]
fn test_execute_success_reference_counts_inliner_0() {
    run_execution_test_success("test_programs/execution_success/reference_counts_inliner_0/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_block_parameter_liveness() {
    run_execution_test_success("test_programs/execution_success/brillig_block_parameter_liveness/Nargo.toml");
}
#[test]
fn test_execute_success_main_return() {
    run_execution_test_success("test_programs/execution_success/main_return/Nargo.toml");
}
#[test]
fn test_execute_success_a_6_array() {
    run_execution_test_success("test_programs/execution_success/a_6_array/Nargo.toml");
}
#[test]
fn test_execute_success_reference_only_used_as_alias() {
    run_execution_test_success("test_programs/execution_success/reference_only_used_as_alias/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8011() {
    run_execution_test_success("test_programs/execution_success/regression_8011/Nargo.toml");
}
#[test]
fn test_execute_success_array_eq() {
    run_execution_test_success("test_programs/execution_success/array_eq/Nargo.toml");
}
#[test]
fn test_execute_success_regression_3607() {
    run_execution_test_success("test_programs/execution_success/regression_3607/Nargo.toml");
}
#[test]
fn test_execute_success_regression_3051() {
    run_execution_test_success("test_programs/execution_success/regression_3051/Nargo.toml");
}
#[test]
fn test_execute_success_regression_4449() {
    run_execution_test_success("test_programs/execution_success/regression_4449/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7836() {
    run_execution_test_success("test_programs/execution_success/regression_7836/Nargo.toml");
}
#[test]
fn test_execute_success_custom_entry() {
    run_execution_test_success("test_programs/execution_success/custom_entry/Nargo.toml");
}
#[test]
fn test_execute_success_a_6() {
    run_execution_test_success("test_programs/execution_success/a_6/Nargo.toml");
}
#[test]
fn test_execute_success_a_3_add() {
    run_execution_test_success("test_programs/execution_success/a_3_add/Nargo.toml");
}
#[test]
fn test_execute_success_type_aliases() {
    run_execution_test_success("test_programs/execution_success/type_aliases/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7451() {
    run_execution_test_success("test_programs/execution_success/regression_7451/Nargo.toml");
}
#[test]
fn test_execute_success_cast_bool() {
    run_execution_test_success("test_programs/execution_success/cast_bool/Nargo.toml");
}
#[test]
fn test_execute_success_shift_right_overflow() {
    run_execution_test_success("test_programs/execution_success/shift_right_overflow/Nargo.toml");
}
#[test]
fn test_execute_success_slices() {
    run_execution_test_success("test_programs/execution_success/slices/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_regression_547() {
    run_execution_test_success("test_programs/execution_success/conditional_regression_547/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_dynamic() {
    run_execution_test_success("test_programs/execution_success/nested_array_dynamic/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6674_3() {
    run_execution_test_success("test_programs/execution_success/regression_6674_3/Nargo.toml");
}
#[test]
fn test_execute_success_regression_struct_array_conditional() {
    run_execution_test_success("test_programs/execution_success/regression_struct_array_conditional/Nargo.toml");
}
#[test]
fn test_execute_success_debug_logs() {
    run_execution_test_success("test_programs/execution_success/debug_logs/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9303() {
    run_execution_test_success("test_programs/execution_success/regression_9303/Nargo.toml");
}
#[test]
fn test_execute_success_struct_array_inputs() {
    run_execution_test_success("test_programs/execution_success/struct_array_inputs/Nargo.toml");
}
#[test]
fn test_execute_success_double_neg_cond_bool_input() {
    run_execution_test_success("test_programs/execution_success/double_neg_cond_bool_input/Nargo.toml");
}
#[test]
fn test_execute_success_negated_jmpif_condition() {
    run_execution_test_success("test_programs/execution_success/negated_jmpif_condition/Nargo.toml");
}
#[test]
fn test_execute_success_bool_or() {
    run_execution_test_success("test_programs/execution_success/bool_or/Nargo.toml");
}
#[test]
fn test_execute_success_lambda_taking_lambda_with_variant() {
    run_execution_test_success("test_programs/execution_success/lambda_taking_lambda_with_variant/Nargo.toml");
}
#[test]
fn test_execute_success_cast_to_i8_regression_7776() {
    run_execution_test_success("test_programs/execution_success/cast_to_i8_regression_7776/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_arrays() {
    run_execution_test_success("test_programs/execution_success/brillig_arrays/Nargo.toml");
}
#[test]
fn test_execute_success_ram_blowup_regression() {
    run_execution_test_success("test_programs/execution_success/ram_blowup_regression/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6674_2() {
    run_execution_test_success("test_programs/execution_success/regression_6674_2/Nargo.toml");
}
#[test]
fn test_execute_success_trait_impl_base_type() {
    run_execution_test_success("test_programs/execution_success/trait_impl_base_type/Nargo.toml");
}
#[test]
fn test_execute_success_a_1327_concrete_in_generic() {
    run_execution_test_success("test_programs/execution_success/a_1327_concrete_in_generic/Nargo.toml");
}
#[test]
fn test_execute_success_simple_not() {
    run_execution_test_success("test_programs/execution_success/simple_not/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_constant_reference_regression() {
    run_execution_test_success("test_programs/execution_success/brillig_constant_reference_regression/Nargo.toml");
}
#[test]
fn test_execute_success_array_with_refs_from_param() {
    run_execution_test_success("test_programs/execution_success/array_with_refs_from_param/Nargo.toml");
}
#[test]
fn test_execute_success_fmtstr_with_global() {
    run_execution_test_success("test_programs/execution_success/fmtstr_with_global/Nargo.toml");
}
#[test]
fn test_execute_success_a_7() {
    run_execution_test_success("test_programs/execution_success/a_7/Nargo.toml");
}
#[test]
fn test_execute_success_blake3() {
    run_execution_test_success("test_programs/execution_success/blake3/Nargo.toml");
}
#[test]
fn test_execute_success_simple_comparison() {
    run_execution_test_success("test_programs/execution_success/simple_comparison/Nargo.toml");
}
#[test]
fn test_execute_success_simple_array_param() {
    run_execution_test_success("test_programs/execution_success/simple_array_param/Nargo.toml");
}
#[test]
fn test_execute_success_inline_never_basic() {
    run_execution_test_success("test_programs/execution_success/inline_never_basic/Nargo.toml");
}
#[test]
fn test_execute_success_to_bytes_integration() {
    run_execution_test_success("test_programs/execution_success/to_bytes_integration/Nargo.toml");
}
#[test]
fn test_execute_success_empty_strings_in_composite_arrays() {
    run_execution_test_success("test_programs/execution_success/empty_strings_in_composite_arrays/Nargo.toml");
}
#[test]
fn test_execute_success_side_effects_constrain_array() {
    run_execution_test_success("test_programs/execution_success/side_effects_constrain_array/Nargo.toml");
}
#[test]
fn test_execute_success_field_attribute() {
    run_execution_test_success("test_programs/execution_success/field_attribute/Nargo.toml");
}
#[test]
fn test_execute_success_signed_division() {
    run_execution_test_success("test_programs/execution_success/signed_division/Nargo.toml");
}
#[test]
fn test_execute_success_references() {
    run_execution_test_success("test_programs/execution_success/references/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_entry_points_regression_8069() {
    run_execution_test_success("test_programs/execution_success/brillig_entry_points_regression_8069/Nargo.toml");
}
#[test]
fn test_execute_success_poseidon_bn254_hash_width_3() {
    run_execution_test_success("test_programs/execution_success/poseidon_bn254_hash_width_3/Nargo.toml");
}
#[test]
fn test_execute_success_fold_2_to_17() {
    run_execution_test_success("test_programs/execution_success/fold_2_to_17/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_with_refs_return() {
    run_execution_test_success("test_programs/execution_success/nested_array_with_refs_return/Nargo.toml");
}
#[test]
fn test_execute_success_fold_after_inlined_calls() {
    run_execution_test_success("test_programs/execution_success/fold_after_inlined_calls/Nargo.toml");
}
#[test]
fn test_execute_success_tuples() {
    run_execution_test_success("test_programs/execution_success/tuples/Nargo.toml");
}
#[test]
fn test_execute_success_regression_mem_op_predicate() {
    run_execution_test_success("test_programs/execution_success/regression_mem_op_predicate/Nargo.toml");
}
#[test]
fn test_execute_success_arithmetic_binary_operations() {
    run_execution_test_success("test_programs/execution_success/arithmetic_binary_operations/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8236() {
    run_execution_test_success("test_programs/execution_success/regression_8236/Nargo.toml");
}
#[test]
fn test_execute_success_missing_closure_env() {
    run_execution_test_success("test_programs/execution_success/missing_closure_env/Nargo.toml");
}
#[test]
fn test_execute_success_ecdsa_secp256r1_invalid_pub_key_in_inactive_branch() {
    run_execution_test_success("test_programs/execution_success/ecdsa_secp256r1_invalid_pub_key_in_inactive_branch/Nargo.toml");
}
#[test]
fn test_execute_success_fold_distinct_return() {
    run_execution_test_success("test_programs/execution_success/fold_distinct_return/Nargo.toml");
}
#[test]
fn test_execute_success_double_neg_cond_global_var() {
    run_execution_test_success("test_programs/execution_success/double_neg_cond_global_var/Nargo.toml");
}
#[test]
fn test_execute_success_wildcard_type() {
    run_execution_test_success("test_programs/execution_success/wildcard_type/Nargo.toml");
}
#[test]
fn test_execute_success_modules_more() {
    run_execution_test_success("test_programs/execution_success/modules_more/Nargo.toml");
}
#[test]
fn test_execute_success_lambda_taking_lambda_regression_8543() {
    run_execution_test_success("test_programs/execution_success/lambda_taking_lambda_regression_8543/Nargo.toml");
}
#[test]
fn test_execute_success_a_7_function() {
    run_execution_test_success("test_programs/execution_success/a_7_function/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8890() {
    run_execution_test_success("test_programs/execution_success/regression_8890/Nargo.toml");
}
#[test]
fn test_execute_success_workspace() {
    run_execution_test_success("test_programs/execution_success/workspace/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_2() {
    run_execution_test_success("test_programs/execution_success/conditional_2/Nargo.toml");
}
#[test]
fn test_execute_success_regression_4202() {
    run_execution_test_success("test_programs/execution_success/regression_4202/Nargo.toml");
}
#[test]
fn test_execute_success_hint_black_box() {
    run_execution_test_success("test_programs/execution_success/hint_black_box/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_fns_as_values() {
    run_execution_test_success("test_programs/execution_success/brillig_fns_as_values/Nargo.toml");
}
#[test]
fn test_execute_success_to_le_bytes() {
    run_execution_test_success("test_programs/execution_success/to_le_bytes/Nargo.toml");
}
#[test]
fn test_execute_success_bit_and() {
    run_execution_test_success("test_programs/execution_success/bit_and/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8662() {
    run_execution_test_success("test_programs/execution_success/regression_8662/Nargo.toml");
}
#[test]
fn test_execute_success_assign_ex() {
    run_execution_test_success("test_programs/execution_success/assign_ex/Nargo.toml");
}
#[test]
fn test_execute_success_ecdsa_secp256k1() {
    run_execution_test_success("test_programs/execution_success/ecdsa_secp256k1/Nargo.toml");
}
#[test]
fn test_execute_success_wrapping_operations() {
    run_execution_test_success("test_programs/execution_success/wrapping_operations/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_not() {
    run_execution_test_success("test_programs/execution_success/brillig_not/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7612() {
    run_execution_test_success("test_programs/execution_success/regression_7612/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6734() {
    run_execution_test_success("test_programs/execution_success/regression_6734/Nargo.toml");
}
#[test]
fn test_execute_success_regression_unused_nested_array_get() {
    run_execution_test_success("test_programs/execution_success/regression_unused_nested_array_get/Nargo.toml");
}
#[test]
fn test_execute_success_a_5_over() {
    run_execution_test_success("test_programs/execution_success/a_5_over/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8726() {
    run_execution_test_success("test_programs/execution_success/regression_8726/Nargo.toml");
}
#[test]
fn test_execute_success_encrypted_log_regression() {
    run_execution_test_success("test_programs/execution_success/encrypted_log_regression/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8980() {
    run_execution_test_success("test_programs/execution_success/regression_8980/Nargo.toml");
}
#[test]
fn test_execute_success_while_loop_break_regression_8521() {
    run_execution_test_success("test_programs/execution_success/while_loop_break_regression_8521/Nargo.toml");
}
#[test]
fn test_execute_success_simple_2d_array() {
    run_execution_test_success("test_programs/execution_success/simple_2d_array/Nargo.toml");
}
#[test]
fn test_execute_success_multi_scalar_mul() {
    run_execution_test_success("test_programs/execution_success/multi_scalar_mul/Nargo.toml");
}
#[test]
fn test_execute_success_closures_mut_ref() {
    run_execution_test_success("test_programs/execution_success/closures_mut_ref/Nargo.toml");
}
#[test]
fn test_execute_success_assert_statement() {
    run_execution_test_success("test_programs/execution_success/assert_statement/Nargo.toml");
}
#[test]
fn test_execute_success_basic() {
    run_execution_test_success("test_programs/execution_success/basic/Nargo.toml");
}
#[test]
fn test_execute_success_array_len() {
    run_execution_test_success("test_programs/execution_success/array_len/Nargo.toml");
}
#[test]
fn test_execute_success_pedersen_commitment() {
    run_execution_test_success("test_programs/execution_success/pedersen_commitment/Nargo.toml");
}
#[test]
fn test_execute_success_unsigned_to_signed_cast() {
    run_execution_test_success("test_programs/execution_success/unsigned_to_signed_cast/Nargo.toml");
}
#[test]
fn test_execute_success_cast_signed_to_u1() {
    run_execution_test_success("test_programs/execution_success/cast_signed_to_u1/Nargo.toml");
}
#[test]
fn test_execute_success_to_be_bytes() {
    run_execution_test_success("test_programs/execution_success/to_be_bytes/Nargo.toml");
}
#[test]
fn test_execute_success_aes128_encrypt() {
    run_execution_test_success("test_programs/execution_success/aes128_encrypt/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9206() {
    run_execution_test_success("test_programs/execution_success/regression_9206/Nargo.toml");
}
#[test]
fn test_execute_success_databus_composite_calldata() {
    run_execution_test_success("test_programs/execution_success/databus_composite_calldata/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_regression_421() {
    run_execution_test_success("test_programs/execution_success/conditional_regression_421/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_cow_assign() {
    run_execution_test_success("test_programs/execution_success/brillig_cow_assign/Nargo.toml");
}
#[test]
fn test_execute_success_fold_call_witness_condition() {
    run_execution_test_success("test_programs/execution_success/fold_call_witness_condition/Nargo.toml");
}
#[test]
fn test_execute_success_array_neq() {
    run_execution_test_success("test_programs/execution_success/array_neq/Nargo.toml");
}
#[test]
fn test_execute_success_regression_3394() {
    run_execution_test_success("test_programs/execution_success/regression_3394/Nargo.toml");
}
#[test]
fn test_execute_success_unrolling_regression_8333() {
    run_execution_test_success("test_programs/execution_success/unrolling_regression_8333/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9208() {
    run_execution_test_success("test_programs/execution_success/regression_9208/Nargo.toml");
}
#[test]
fn test_execute_success_signed_truncation() {
    run_execution_test_success("test_programs/execution_success/signed_truncation/Nargo.toml");
}
#[test]
fn test_execute_success_array_oob_regression_7965() {
    run_execution_test_success("test_programs/execution_success/array_oob_regression_7965/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8329() {
    run_execution_test_success("test_programs/execution_success/regression_8329/Nargo.toml");
}
#[test]
fn test_execute_success_break_and_continue() {
    run_execution_test_success("test_programs/execution_success/break_and_continue/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8729() {
    run_execution_test_success("test_programs/execution_success/regression_8729/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8975() {
    run_execution_test_success("test_programs/execution_success/regression_8975/Nargo.toml");
}
#[test]
fn test_execute_success_array_dedup_regression() {
    run_execution_test_success("test_programs/execution_success/array_dedup_regression/Nargo.toml");
}
#[test]
fn test_execute_success_pedersen_hash() {
    run_execution_test_success("test_programs/execution_success/pedersen_hash/Nargo.toml");
}
#[test]
fn test_execute_success_reference_counts_inliner_min() {
    run_execution_test_success("test_programs/execution_success/reference_counts_inliner_min/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_identity_function() {
    run_execution_test_success("test_programs/execution_success/brillig_identity_function/Nargo.toml");
}
#[test]
fn test_execute_success_loop_invariant_regression_8586() {
    run_execution_test_success("test_programs/execution_success/loop_invariant_regression_8586/Nargo.toml");
}
#[test]
fn test_execute_success_assert() {
    run_execution_test_success("test_programs/execution_success/assert/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_calls_array() {
    run_execution_test_success("test_programs/execution_success/brillig_calls_array/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8926() {
    run_execution_test_success("test_programs/execution_success/regression_8926/Nargo.toml");
}
#[test]
fn test_execute_success_slice_loop() {
    run_execution_test_success("test_programs/execution_success/slice_loop/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_cow_regression() {
    run_execution_test_success("test_programs/execution_success/brillig_cow_regression/Nargo.toml");
}
#[test]
fn test_execute_success_fold_fibonacci() {
    run_execution_test_success("test_programs/execution_success/fold_fibonacci/Nargo.toml");
}
#[test]
fn test_execute_success_no_predicates_numeric_generic_poseidon() {
    run_execution_test_success("test_programs/execution_success/no_predicates_numeric_generic_poseidon/Nargo.toml");
}
#[test]
fn test_execute_success_hashmap() {
    run_execution_test_success("test_programs/execution_success/hashmap/Nargo.toml");
}
#[test]
fn test_execute_success_array_with_refs_return() {
    run_execution_test_success("test_programs/execution_success/array_with_refs_return/Nargo.toml");
}
#[test]
fn test_execute_success_binary_operator_overloading() {
    run_execution_test_success("test_programs/execution_success/binary_operator_overloading/Nargo.toml");
}
#[test]
fn test_execute_success_array_sort() {
    run_execution_test_success("test_programs/execution_success/array_sort/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9243() {
    run_execution_test_success("test_programs/execution_success/regression_9243/Nargo.toml");
}
#[test]
fn test_execute_success_regression_2660() {
    run_execution_test_success("test_programs/execution_success/regression_2660/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_regression_underflow() {
    run_execution_test_success("test_programs/execution_success/conditional_regression_underflow/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_cow() {
    run_execution_test_success("test_programs/execution_success/brillig_cow/Nargo.toml");
}
#[test]
fn test_execute_success_nested_arrays_from_brillig() {
    run_execution_test_success("test_programs/execution_success/nested_arrays_from_brillig/Nargo.toml");
}
#[test]
fn test_execute_success_regression_11294() {
    run_execution_test_success("test_programs/execution_success/regression_11294/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8739() {
    run_execution_test_success("test_programs/execution_success/regression_8739/Nargo.toml");
}
#[test]
fn test_execute_success_global_array_rc_regression_8259() {
    run_execution_test_success("test_programs/execution_success/global_array_rc_regression_8259/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6834() {
    run_execution_test_success("test_programs/execution_success/regression_6834/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_blake2s() {
    run_execution_test_success("test_programs/execution_success/brillig_blake2s/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7323() {
    run_execution_test_success("test_programs/execution_success/regression_7323/Nargo.toml");
}
#[test]
fn test_execute_success_struct() {
    run_execution_test_success("test_programs/execution_success/struct/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9047() {
    run_execution_test_success("test_programs/execution_success/regression_9047/Nargo.toml");
}
#[test]
fn test_execute_success_regression_5615() {
    run_execution_test_success("test_programs/execution_success/regression_5615/Nargo.toml");
}
#[test]
fn test_execute_success_global_var_regression_entry_points() {
    run_execution_test_success("test_programs/execution_success/global_var_regression_entry_points/Nargo.toml");
}
#[test]
fn test_execute_success_to_bytes_consistent() {
    run_execution_test_success("test_programs/execution_success/to_bytes_consistent/Nargo.toml");
}
#[test]
fn test_execute_success_lambda_from_global_tuple() {
    run_execution_test_success("test_programs/execution_success/lambda_from_global_tuple/Nargo.toml");
}
#[test]
fn test_execute_success_lambda_from_global_array() {
    run_execution_test_success("test_programs/execution_success/lambda_from_global_array/Nargo.toml");
}
#[test]
fn test_execute_success_derive() {
    run_execution_test_success("test_programs/execution_success/derive/Nargo.toml");
}
#[test]
fn test_execute_success_array_to_slice() {
    run_execution_test_success("test_programs/execution_success/array_to_slice/Nargo.toml");
}
#[test]
fn test_execute_success_signed_cmp() {
    run_execution_test_success("test_programs/execution_success/signed_cmp/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8558() {
    run_execution_test_success("test_programs/execution_success/regression_8558/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8755() {
    run_execution_test_success("test_programs/execution_success/regression_8755/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7128() {
    run_execution_test_success("test_programs/execution_success/regression_7128/Nargo.toml");
}
#[test]
fn test_execute_success_traits_in_crates_1() {
    run_execution_test_success("test_programs/execution_success/traits_in_crates_1/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_regression_661() {
    run_execution_test_success("test_programs/execution_success/conditional_regression_661/Nargo.toml");
}
#[test]
fn test_execute_success_bit_shifts_runtime() {
    run_execution_test_success("test_programs/execution_success/bit_shifts_runtime/Nargo.toml");
}
#[test]
fn test_execute_success_regression_capacity_tracker() {
    run_execution_test_success("test_programs/execution_success/regression_capacity_tracker/Nargo.toml");
}
#[test]
fn test_execute_success_struct_assignment_with_shared_ref_to_field() {
    run_execution_test_success("test_programs/execution_success/struct_assignment_with_shared_ref_to_field/Nargo.toml");
}
#[test]
fn test_execute_success_nested_fmtstr() {
    run_execution_test_success("test_programs/execution_success/nested_fmtstr/Nargo.toml");
}
#[test]
fn test_execute_success_a_1_mul() {
    run_execution_test_success("test_programs/execution_success/a_1_mul/Nargo.toml");
}
#[test]
fn test_execute_success_a_2_div() {
    run_execution_test_success("test_programs/execution_success/a_2_div/Nargo.toml");
}
#[test]
fn test_execute_success_overlapping_dep_and_mod() {
    run_execution_test_success("test_programs/execution_success/overlapping_dep_and_mod/Nargo.toml");
}
#[test]
fn test_execute_success_bit_shifts_comptime() {
    run_execution_test_success("test_programs/execution_success/bit_shifts_comptime/Nargo.toml");
}
#[test]
fn test_execute_success_witness_compression() {
    run_execution_test_success("test_programs/execution_success/witness_compression/Nargo.toml");
}
#[test]
fn test_execute_success_trait_associated_constant() {
    run_execution_test_success("test_programs/execution_success/trait_associated_constant/Nargo.toml");
}
#[test]
fn test_execute_success_regression_1144_1169_2399_6609() {
    run_execution_test_success("test_programs/execution_success/regression_1144_1169_2399_6609/Nargo.toml");
}
#[test]
fn test_execute_success_if_else_chain() {
    run_execution_test_success("test_programs/execution_success/if_else_chain/Nargo.toml");
}
#[test]
fn test_execute_success_regression_5045() {
    run_execution_test_success("test_programs/execution_success/regression_5045/Nargo.toml");
}
#[test]
fn test_execute_success_bool_not() {
    run_execution_test_success("test_programs/execution_success/bool_not/Nargo.toml");
}
#[test]
fn test_execute_success_struct_fields_ordering() {
    run_execution_test_success("test_programs/execution_success/struct_fields_ordering/Nargo.toml");
}
#[test]
fn test_execute_success_nested_dyn_array_regression_5782() {
    run_execution_test_success("test_programs/execution_success/nested_dyn_array_regression_5782/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7143() {
    run_execution_test_success("test_programs/execution_success/regression_7143/Nargo.toml");
}
#[test]
fn test_execute_success_ecdsa_secp256r1() {
    run_execution_test_success("test_programs/execution_success/ecdsa_secp256r1/Nargo.toml");
}
#[test]
fn test_execute_success_array_oob_regression_7975() {
    run_execution_test_success("test_programs/execution_success/array_oob_regression_7975/Nargo.toml");
}
#[test]
fn test_execute_success_array_dynamic() {
    run_execution_test_success("test_programs/execution_success/array_dynamic/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9160() {
    run_execution_test_success("test_programs/execution_success/regression_9160/Nargo.toml");
}
#[test]
fn test_execute_success_merkle_insert() {
    run_execution_test_success("test_programs/execution_success/merkle_insert/Nargo.toml");
}
#[test]
fn test_execute_success_cast_to_u128() {
    run_execution_test_success("test_programs/execution_success/cast_to_u128/Nargo.toml");
}
#[test]
fn test_execute_success_slice_coercion() {
    run_execution_test_success("test_programs/execution_success/slice_coercion/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9193() {
    run_execution_test_success("test_programs/execution_success/regression_9193/Nargo.toml");
}
#[test]
fn test_execute_success_array_dynamic_nested_blackbox_input() {
    run_execution_test_success("test_programs/execution_success/array_dynamic_nested_blackbox_input/Nargo.toml");
}
#[test]
fn test_execute_success_diamond_deps_0() {
    run_execution_test_success("test_programs/execution_success/diamond_deps_0/Nargo.toml");
}
#[test]
fn test_execute_success_simple_mut() {
    run_execution_test_success("test_programs/execution_success/simple_mut/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8874() {
    run_execution_test_success("test_programs/execution_success/regression_8874/Nargo.toml");
}
#[test]
fn test_execute_success_regression_unsafe_no_predicates() {
    run_execution_test_success("test_programs/execution_success/regression_unsafe_no_predicates/Nargo.toml");
}
#[test]
fn test_execute_success_poseidonsponge_x5_254() {
    run_execution_test_success("test_programs/execution_success/poseidonsponge_x5_254/Nargo.toml");
}
#[test]
fn test_execute_success_no_predicates_basic() {
    run_execution_test_success("test_programs/execution_success/no_predicates_basic/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9102() {
    run_execution_test_success("test_programs/execution_success/regression_9102/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_conditional() {
    run_execution_test_success("test_programs/execution_success/brillig_conditional/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7062() {
    run_execution_test_success("test_programs/execution_success/regression_7062/Nargo.toml");
}
#[test]
fn test_execute_success_submodules() {
    run_execution_test_success("test_programs/execution_success/submodules/Nargo.toml");
}
#[test]
fn test_execute_success_loop_break_regression_8319() {
    run_execution_test_success("test_programs/execution_success/loop_break_regression_8319/Nargo.toml");
}
#[test]
fn test_execute_success_regression_6674_1() {
    run_execution_test_success("test_programs/execution_success/regression_6674_1/Nargo.toml");
}
#[test]
fn test_execute_success_array_to_slice_constant_length() {
    run_execution_test_success("test_programs/execution_success/array_to_slice_constant_length/Nargo.toml");
}
#[test]
fn test_execute_success_fold_basic_nested_call() {
    run_execution_test_success("test_programs/execution_success/fold_basic_nested_call/Nargo.toml");
}
#[test]
fn test_execute_success_higher_order_functions() {
    run_execution_test_success("test_programs/execution_success/higher_order_functions/Nargo.toml");
}
#[test]
fn test_execute_success_loop_small_break() {
    run_execution_test_success("test_programs/execution_success/loop_small_break/Nargo.toml");
}
#[test]
fn test_execute_success_ecdsa_secp256k1_invalid_pub_key_in_inactive_branch() {
    run_execution_test_success("test_programs/execution_success/ecdsa_secp256k1_invalid_pub_key_in_inactive_branch/Nargo.toml");
}
#[test]
fn test_execute_success_uhashmap() {
    run_execution_test_success("test_programs/execution_success/uhashmap/Nargo.toml");
}
#[test]
fn test_execute_success_regression_4088() {
    run_execution_test_success("test_programs/execution_success/regression_4088/Nargo.toml");
}
#[test]
fn test_execute_success_signed_div() {
    run_execution_test_success("test_programs/execution_success/signed_div/Nargo.toml");
}
#[test]
fn test_execute_success_signed_arithmetic() {
    run_execution_test_success("test_programs/execution_success/signed_arithmetic/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_regression_short_circuit() {
    run_execution_test_success("test_programs/execution_success/conditional_regression_short_circuit/Nargo.toml");
}
#[test]
fn test_execute_success_return_twice() {
    run_execution_test_success("test_programs/execution_success/return_twice/Nargo.toml");
}
#[test]
fn test_execute_success_signed_overflow_in_else_regression_8617() {
    run_execution_test_success("test_programs/execution_success/signed_overflow_in_else_regression_8617/Nargo.toml");
}
#[test]
fn test_execute_success_global_var_entry_point_used_in_another_entry() {
    run_execution_test_success("test_programs/execution_success/global_var_entry_point_used_in_another_entry/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8235() {
    run_execution_test_success("test_programs/execution_success/regression_8235/Nargo.toml");
}
#[test]
fn test_execute_success_tuple_inputs() {
    run_execution_test_success("test_programs/execution_success/tuple_inputs/Nargo.toml");
}
#[test]
fn test_execute_success_function_ref() {
    run_execution_test_success("test_programs/execution_success/function_ref/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9119() {
    run_execution_test_success("test_programs/execution_success/regression_9119/Nargo.toml");
}
#[test]
fn test_execute_success_simple_print() {
    run_execution_test_success("test_programs/execution_success/simple_print/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8009() {
    run_execution_test_success("test_programs/execution_success/regression_8009/Nargo.toml");
}
#[test]
fn test_execute_success_global_var_multiple_entry_points_nested() {
    run_execution_test_success("test_programs/execution_success/global_var_multiple_entry_points_nested/Nargo.toml");
}
#[test]
fn test_execute_success_u128_type() {
    run_execution_test_success("test_programs/execution_success/u128_type/Nargo.toml");
}
#[test]
fn test_execute_success_embedded_curve_ops() {
    run_execution_test_success("test_programs/execution_success/embedded_curve_ops/Nargo.toml");
}
#[test]
fn test_execute_success_nested_if_then_block_same_cond() {
    run_execution_test_success("test_programs/execution_success/nested_if_then_block_same_cond/Nargo.toml");
}
#[test]
fn test_execute_success_main_bool_arg() {
    run_execution_test_success("test_programs/execution_success/main_bool_arg/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_nested_arrays() {
    run_execution_test_success("test_programs/execution_success/brillig_nested_arrays/Nargo.toml");
}
#[test]
fn test_execute_success_slice_regex() {
    run_execution_test_success("test_programs/execution_success/slice_regex/Nargo.toml");
}
#[test]
fn test_execute_success_cast_to_u64_regression_7776() {
    run_execution_test_success("test_programs/execution_success/cast_to_u64_regression_7776/Nargo.toml");
}
#[test]
fn test_execute_success_signed_comparison() {
    run_execution_test_success("test_programs/execution_success/signed_comparison/Nargo.toml");
}
#[test]
fn test_execute_success_unary_operator_overloading() {
    run_execution_test_success("test_programs/execution_success/unary_operator_overloading/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8261() {
    run_execution_test_success("test_programs/execution_success/regression_8261/Nargo.toml");
}
#[test]
fn test_execute_success_comptime_println() {
    run_execution_test_success("test_programs/execution_success/comptime_println/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9116() {
    run_execution_test_success("test_programs/execution_success/regression_9116/Nargo.toml");
}
#[test]
fn test_execute_success_array_if_cond_simple() {
    run_execution_test_success("test_programs/execution_success/array_if_cond_simple/Nargo.toml");
}
#[test]
fn test_execute_success_databus_two_calldata() {
    run_execution_test_success("test_programs/execution_success/databus_two_calldata/Nargo.toml");
}
#[test]
fn test_execute_success_unsafe_range_constraint() {
    run_execution_test_success("test_programs/execution_success/unsafe_range_constraint/Nargo.toml");
}
#[test]
fn test_execute_success_prelude() {
    run_execution_test_success("test_programs/execution_success/prelude/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9312() {
    run_execution_test_success("test_programs/execution_success/regression_9312/Nargo.toml");
}
#[test]
fn test_execute_success_comptime_variable_at_runtime() {
    run_execution_test_success("test_programs/execution_success/comptime_variable_at_runtime/Nargo.toml");
}
#[test]
fn test_execute_success_regression_3889() {
    run_execution_test_success("test_programs/execution_success/regression_3889/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_calls() {
    run_execution_test_success("test_programs/execution_success/brillig_calls/Nargo.toml");
}
#[test]
fn test_execute_success_modules() {
    run_execution_test_success("test_programs/execution_success/modules/Nargo.toml");
}
#[test]
fn test_execute_success_workspace_default_member() {
    run_execution_test_success("test_programs/execution_success/workspace_default_member/Nargo.toml");
}
#[test]
fn test_execute_success_import() {
    run_execution_test_success("test_programs/execution_success/import/Nargo.toml");
}
#[test]
fn test_execute_success_reference_counts_slices_inliner_0() {
    run_execution_test_success("test_programs/execution_success/reference_counts_slices_inliner_0/Nargo.toml");
}
#[test]
fn test_execute_success_bench_ecdsa_secp256k1() {
    run_execution_test_success("test_programs/execution_success/bench_ecdsa_secp256k1/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_calls_conditionals() {
    run_execution_test_success("test_programs/execution_success/brillig_calls_conditionals/Nargo.toml");
}
#[test]
fn test_execute_success_inline_decompose_hint_brillig_call() {
    run_execution_test_success("test_programs/execution_success/inline_decompose_hint_brillig_call/Nargo.toml");
}
#[test]
fn test_execute_success_slice_dynamic_index() {
    run_execution_test_success("test_programs/execution_success/slice_dynamic_index/Nargo.toml");
}
#[test]
fn test_execute_success_conditional_1() {
    run_execution_test_success("test_programs/execution_success/conditional_1/Nargo.toml");
}
#[test]
fn test_execute_success_static_assert_empty_loop() {
    run_execution_test_success("test_programs/execution_success/static_assert_empty_loop/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7962() {
    run_execution_test_success("test_programs/execution_success/regression_7962/Nargo.toml");
}
#[test]
fn test_execute_success_signed_inactive_division_by_zero() {
    run_execution_test_success("test_programs/execution_success/signed_inactive_division_by_zero/Nargo.toml");
}
#[test]
fn test_execute_success_regression_7195() {
    run_execution_test_success("test_programs/execution_success/regression_7195/Nargo.toml");
}
#[test]
fn test_execute_success_regression_5252() {
    run_execution_test_success("test_programs/execution_success/regression_5252/Nargo.toml");
}
#[test]
fn test_execute_success_databus_two_calldata_simple() {
    run_execution_test_success("test_programs/execution_success/databus_two_calldata_simple/Nargo.toml");
}
#[test]
fn test_execute_success_regression_9037() {
    run_execution_test_success("test_programs/execution_success/regression_9037/Nargo.toml");
}
#[test]
fn test_execute_success_simple_program_addition() {
    run_execution_test_success("test_programs/execution_success/simple_program_addition/Nargo.toml");
}
#[test]
fn test_execute_success_bit_not() {
    run_execution_test_success("test_programs/execution_success/bit_not/Nargo.toml");
}
#[test]
fn test_execute_success_regression_5435() {
    run_execution_test_success("test_programs/execution_success/regression_5435/Nargo.toml");
}
#[test]
fn test_execute_success_no_predicates_brillig() {
    run_execution_test_success("test_programs/execution_success/no_predicates_brillig/Nargo.toml");
}
#[test]
fn test_execute_success_fold_numeric_generic_poseidon() {
    run_execution_test_success("test_programs/execution_success/fold_numeric_generic_poseidon/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_acir_as_brillig() {
    run_execution_test_success("test_programs/execution_success/brillig_acir_as_brillig/Nargo.toml");
}
#[test]
fn test_execute_success_pred_eq() {
    run_execution_test_success("test_programs/execution_success/pred_eq/Nargo.toml");
}
#[test]
fn test_execute_success_regression_4124() {
    run_execution_test_success("test_programs/execution_success/regression_4124/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8174() {
    run_execution_test_success("test_programs/execution_success/regression_8174/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_loop_size_regression() {
    run_execution_test_success("test_programs/execution_success/brillig_loop_size_regression/Nargo.toml");
}
#[test]
fn test_execute_success_loop_invariant_nested_deep() {
    run_execution_test_success("test_programs/execution_success/loop_invariant_nested_deep/Nargo.toml");
}
#[test]
fn test_execute_success_u16_support() {
    run_execution_test_success("test_programs/execution_success/u16_support/Nargo.toml");
}
#[test]
fn test_execute_success_lambda_from_dynamic_if() {
    run_execution_test_success("test_programs/execution_success/lambda_from_dynamic_if/Nargo.toml");
}
#[test]
fn test_execute_success_simple_add_and_ret_arr() {
    run_execution_test_success("test_programs/execution_success/simple_add_and_ret_arr/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_recursion() {
    run_execution_test_success("test_programs/execution_success/brillig_recursion/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8519() {
    run_execution_test_success("test_programs/execution_success/regression_8519/Nargo.toml");
}
#[test]
fn test_execute_success_brillig_uninitialized_arrays() {
    run_execution_test_success("test_programs/execution_success/brillig_uninitialized_arrays/Nargo.toml");
}
#[test]
fn test_execute_success_struct_inputs() {
    run_execution_test_success("test_programs/execution_success/struct_inputs/Nargo.toml");
}
#[test]
fn test_execute_success_databus() {
    run_execution_test_success("test_programs/execution_success/databus/Nargo.toml");
}
#[test]
fn test_execute_success_loop() {
    run_execution_test_success("test_programs/execution_success/loop/Nargo.toml");
}
#[test]
fn test_execute_success_global_slice_rc_regression_8259() {
    run_execution_test_success("test_programs/execution_success/global_slice_rc_regression_8259/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_with_refs_from_param() {
    run_execution_test_success("test_programs/execution_success/nested_array_with_refs_from_param/Nargo.toml");
}
#[test]
fn test_execute_success_cast_to_u8_regression_7776() {
    run_execution_test_success("test_programs/execution_success/cast_to_u8_regression_7776/Nargo.toml");
}
#[test]
fn test_execute_success_regression_8779() {
    run_execution_test_success("test_programs/execution_success/regression_8779/Nargo.toml");
}
#[test]
fn test_execute_success_trait_as_return_type() {
    run_execution_test_success("test_programs/execution_success/trait_as_return_type/Nargo.toml");
}
#[test]
fn test_execute_success_nested_array_dynamic_simple() {
    run_execution_test_success("test_programs/execution_success/nested_array_dynamic_simple/Nargo.toml");
}
#[test]
fn test_execute_success_global_var_func_with_multiple_entry_points() {
    run_execution_test_success("test_programs/execution_success/global_var_func_with_multiple_entry_points/Nargo.toml");
}
#[test]
fn test_execute_success_regression_method_cannot_be_found() {
    run_execution_test_success("test_programs/execution_success/regression_method_cannot_be_found/Nargo.toml");
}
#[test]
fn test_execute_success_array_dynamic_blackbox_input() {
    run_execution_test_success("test_programs/execution_success/array_dynamic_blackbox_input/Nargo.toml");
}
#[test]
fn test_execute_success_bench_2_to_17() {
    run_execution_test_success("test_programs/execution_success/bench_2_to_17/Nargo.toml");
}
#[test]
fn test_execute_success_fold_complex_outputs() {
    run_execution_test_success("test_programs/execution_success/fold_complex_outputs/Nargo.toml");
}
#[test]
fn test_execute_success_array_dynamic_main_output() {
    run_execution_test_success("test_programs/execution_success/array_dynamic_main_output/Nargo.toml");
}
#[test]
fn test_execute_success_global_consts() {
    run_execution_test_success("test_programs/execution_success/global_consts/Nargo.toml");
}