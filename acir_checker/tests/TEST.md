# Testing NAVe

Given that the problem that NAVe solves (i.e. checking satisfiability for a set of polynomial equations over a finite field)
is very computationally intensive, some of the test cases can take a little too long and timeout. For that reason, we use cargo [nextest](https://nexte.st/) to carry out our tests with a timeout; [installation instructions here](https://nexte.st/docs/installation/pre-built-binaries/).

* `failures.rs`: tests the executions in `test_programs/execution_failures` and ensure that they do
not lead to a satisfiable assignment. By testing an execution, we mean that we analyse whether the `Provel.toml` input (and maybe output) values can lead to a valid execution of the Noir program. That is, whether our Noir SMT constraints combined with the assignment of the input/output values to the corresponding input/output variables is satisfiable. An *execution failure* means that there is no satisfiable assignment for these SMT constraints. An *execution success* means that an assignment *does exist*. The test harness allows the test to timeout and this will not be considered a failure. A test failure is only detected if the constraints are found to be satisfiable, representing an execution success as opposed to the expected failure. These programs have been curated by the Noir development team.
* `success.rs`: tests the executions in `test_programs/execution_successes` and ensure that they lead to a satisfiable assignment. The counterpart/dual to the examples above. These programs have been curated by the Noir development team.
* `test_falsified.rs`: has a set of Noir programs, created by us, with verification assertions that fail.
* `test_verified.rs`: has a set of Noir programs, created by us, with verification assertions that fail.

