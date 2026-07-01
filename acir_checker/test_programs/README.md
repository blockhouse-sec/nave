# Programs omitted from tests

The following examples have been removed in `execution_failure` because they lead to
trivially satisfiable constraints. They focus on unconstrained
code execution errors, which we are not interested in for the
moment.

We have removed:
- brillig_mem_layout_regression
- brillig_entry_points_shared_recursive
- mutually_recursive_simple_functions
- regression_10238 (missing argument on Prover.toml)
- regression_9904
- shl_signed_regression_9592
- simple_infinite_recursive_function
- simple_infinite_recursive_lambda
- workspace_fail

## Programs with no constraints

The following programs compile to an ACIR circuit with no constraints (verified
via `nargo compile --print-acir`: the circuit body contains no `ASSERT`
opcodes, only `BRILLIG CALL`s or nothing at all). Since there are no
constraints to check, they are omitted from the results reported in the
paper.

`execution_failure`:
- mocks_in_execution
- regression_claude_1303
- unknown_oracle

`execution_success`:
- array_dedup_regression
- array_rc_regression_7842
- array_set_not_deduplicated
- array_to_vector_constant_length
- array_with_refs_from_param
- array_with_refs_return
- as_str_unchecked_with_broken_bytes
- break_and_continue
- brillig_array_input_indirectly_mutated
- brillig_arrays
- brillig_block_parameter_liveness
- brillig_constant_reference_regression
- brillig_cow
- brillig_cow_assign
- brillig_cow_regression
- brillig_if_mutable_reference_regression
- brillig_large_array
- brillig_large_nested_array
- brillig_loop_size_regression
- brillig_pedersen
- brillig_rc_regression_6123
- brillig_recursive_main
- brillig_recursive_main_indirect
- clone_index_field_dereference
- clone_index_object_dereference_1
- clone_index_object_dereference_2
- comptime_closure_bindings_1
- comptime_closure_bindings_2
- comptime_generics_binding
- comptime_println
- comptime_println_fmtstr_with_quoted
- comptime_quoted_hash
- comptime_trait_constraint_hash_and_eq
- comptime_variable_at_runtime
- constant_folding_mutated_returned_array_bug
- debug_name_no_conflict
- derive
- do_not_capture_comptime_locals
- double_neg_cond_bool_input
- dual_constrained_lambdas
- empty_strings_in_composite_arrays
- fmtstr_with_global
- for_loop_inclusive_empty_range
- for_loop_inclusive_u8_max
- for_loop_inclusive_with_break
- global_var_entry_point_used_in_another_entry
- global_var_func_with_multiple_entry_points
- global_var_multiple_entry_points_nested
- lambda_env_is_copied
- lambda_taking_lambda_regression_8543
- lambda_taking_lambda_with_variant
- last_uses_regression_8935
- licm_bug_inverted_loop
- local_module_does_not_conflict_with_debugger
- loop_break_regression_8319
- loop_small_break
- multi_scalar_mul
- mutable_and_immutable_reference_alias
- mutate_array_copy
- negative_associated_constants
- nested_array_index_clone_regression
- nested_array_with_refs_from_param
- nested_array_with_refs_return
- nested_fmtstr
- no_predicates_brillig
- op_assign_desugaring
- print_composite_array
- reference_alias_in_array
- reference_cancelling
- reference_counts_inliner_0
- reference_counts_inliner_max
- reference_counts_inliner_min
- regression_10158
- regression_10452
- regression_10466
- regression_10516
- regression_10690
- regression_10917
- regression_11294
- regression_11440
- regression_11463
- regression_11484
- regression_11540
- regression_12149
- regression_12269
- regression_12317
- regression_12475
- regression_12713
- regression_3051
- regression_3394
- regression_4663
- regression_5615
- regression_6674_1
- regression_6674_2
- regression_6674_3
- regression_6734
- regression_6990
- regression_8009
- regression_8011
- regression_8174
- regression_8739
- regression_8755
- regression_8926
- regression_8980
- regression_9037
- regression_9116
- regression_9119
- regression_9243
- regression_9294
- regression_9303
- regression_9415
- regression_9439
- regression_9455
- regression_9538
- regression_9578
- regression_9657
- regression_9725_1
- regression_9725_2
- regression_9758
- regression_brillig_const_fold_self_dedup
- regression_licm_induction_var
- regression_mem2reg_unknown_array_aliases
- regression_method_cannot_be_found
- regression_unroll_body_break
- regression_unused_nested_array_get
- signed_bitshift
- simple_print
- static_assert_empty_loop
- struct_assignment_with_shared_ref_to_field
- trait_associated_constant
- uhashmap
- unroll_loop_regression
- vector_pop_front_aliased_source
- vector_regex
- while_cond_clone_regression
- while_loop_break_regression_8521