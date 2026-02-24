The following examples have been removed in `execution_failure` because they lead to
trivially satisfiable constraints. They focus on unconstrained
code execution errors, which we are not interested in for the
moment.

We have removed:
- brillig_mem_layout_regression
- brillig_entry_points_shared_recursive
- mutually_recursive_simple_functions
- regression_10238 (missing argument on Prover.toml)
- simple_infinite_recursive_function
- simple_infinite_recursive_lambda
- workspace_fail