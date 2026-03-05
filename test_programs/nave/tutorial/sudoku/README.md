# Formally verifying Sudoku in NAVe

## Why Sudoku in Zero Knowledge?

In a zero-knowledge setting, we want to prove that:

“Sudoku was solved correctly without revealing the actual solution.”

4*4 Sudoku is a perfect minimal example because:

1. Inputs are private.
2. Rules are simple but non-trivial.

Although real Sudoku uses a 9×9 grid, the 4×4 variant preserves the logical properties required for formal reasoning while keeping the circuit compact.

## Why This Is Formally Verifiable

A Sudoku solution is valid if:

1. Every value lies within an allowed range.
2. Every row, column, and subgrid contains each value exactly once.

These properties make the system well-suited to formal verification because:

1. Every rule can be modelled as a constraint
2. Execution is deterministic (no non-deterministic branching)
3. Invalid states are unreachable once constraints hold

Thus the verification problem reduces to checking that no counterexample board exists that satisfies the circuit while
violating Sudoku rules.

## ZK Verification Model

We have one private witness:

4 * 4 Sudoku board with `Field` values denoting the solution.

We can assert that:

1. Each value in the board is valid (1-4)
2. Every row, column and 2*2 subgrid contains all values exactly once.

The circuit ultimately computes a boolean result indicating whether the board satisfies all constraints.

## Modeling Sudoku in Noir

In Noir, we encode the sudoku board values (`val`) as field elements.

We have explicitly constrained the input values to be valid in Noir.

```
assert((val == 1) | (val == 2) | (val == 3) | (val == 4));
```

Though there is a relatively easy way to constrain this via modelling the values as `u8` (highlighted below) but it generates `BLACKBOX::RANGE` opcode in ACIR which is hard to evaluate.

```
assert((val as u8) <= 4);
```

## Formal Verification Conditions (Explicit) and Analysis

The following constraints formally specify and verify the correctness of the solution.

The circuit uses `verify_assert` statements to encode logical properties that must hold for all possible inputs. A verification result of `Verified` means that no counterexample exists, i.e., the negation of the asserted properties is unsatisfiable (UNSAT).

In this section, we only focus on checking the correctness of Sudoku columns:

```
// Columns
for c in 0..4 {
    let mut seen = [false; 4];
    for r in 0..4 {
        let index = (board[r][c] - 1) as u32;
        assert(!seen[index]);
        seen[index] = true;
    }
}
```

The algorithm works as follows:

1. Iterate through each column `c`.
2. Maintain a seen array representing whether a value {1..4} has already appeared.
3. Convert the cell value into an index `(value - 1)`.
4. Assert that the value has not been seen previously.
5. Mark the value as seen.

If any value appears twice in the same column, the assertion fails.

Main entry point:

```
fn main(board: [[Field; 4]; 4]) -> pub bool {
    let solution = verify_sudoku(board);
    unsafe { verify_assert(solution); }
    solution
}
```

`verify_assert(solution)` encodes the property that the result must always be true.
If the verifier reports Verified, it means the solver has proven that:

For all valid inputs to the circuit, the Sudoku verification logic cannot evaluate to false.

## How to run the verifier?

To run the verifier, from `sudoku` directory, use the following command:

`cargo run -- formal-verify`

We can also specify configuration options to it -- more details in `tooling/acir_checker/README.md`
