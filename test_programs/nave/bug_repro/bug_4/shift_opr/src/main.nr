//! Bug: https://github.com/noir-lang/noir/issues/7412
//!      https://github.com/noir-lang/noir/issues/9022
//! Fix: https://github.com/noir-lang/noir/pull/7509
//! Latest related fix: https://github.com/noir-lang/noir/pull/9373
//!
//! The verifier can demonstrate inconsistencies in how right-shift
//! operations are modeled when the shift amount exceeds bit-width of 
//! the operand.
//!
//! Currently, times-out with range checks enabled.
//! Otherwise, it falsifies the assertion (i.e. `value1` is not equal to `value2`) 
//! with a very big return value (not fitting into `u8`), though the expected 
//! behavior is that both `value1` and `value2` should be zero for shift amounts >= 8.

fn main(x: u8) -> pub u8 {
    // Placing this assertion earlier (before the shift operations) triggers
    // a Noir bug or total simplification of ACIR, so intentionally left here.
    // assert(x == 8);

    let value1 = 1 >> x;

    // Using a literal shift amount (`1 >> 8`) triggers a Noir bug,
    // whereas an equivalent computed value (e.g. `x + 1` with x == 7)
    // does not exhibit the same behavior.
    // let value2 = 1 >> 8;
    let value2 = 1 >> (x + 1);

    // This assertion behaves inconsistently:
    // - For x >= 8, execution terminates early - reporting it as bug.
    // - For x < 8, execution continues/times out.
    //
    // Shift operation introduces an implicit constraint (x < 8 for u8)
    // which affects control flow and assertion handling.
    assert(x == 8);

    // Timeout with range checks enabled, Falsified otherwise.
    unsafe { verify_assert(value1 == value2); } 

    // The following variants lead to solver timeouts, likely due to the
    // complexity of the generated constraints:
    // unsafe { verify_assert(((x == 8) | (x == 9)) & (value1 == 0)); }
    // unsafe { verify_assert(!((x == 0) | (x == 1) | (x == 2) | (x == 3)) & (value1 == 0)); }

    value1 + value2
}

unconstrained fn verify_assert(b: bool) { assert(b); }