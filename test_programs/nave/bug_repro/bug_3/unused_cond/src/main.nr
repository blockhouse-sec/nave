//! Bug: https://github.com/noir-lang/noir/issues/8235
//!
//! The program below failed in ACIR but succeeded in Brillig.
//! This appears to be due to incorrect handling of the predicate
//! in Euclidean division, specifically when dividing by zero under
//! a non-constant predicate.
//!
//! Fix by Noir team: https://github.com/noir-lang/noir/pull/8243
//!
//! With our changes, the solver can determine that there are cases
//! where ACIR does not need to evaluate the division. In such cases,
//! it can safely return alternative values, aligning the behavior of
//! both ACIR and Brillig for the function `func`.

fn main(a: bool, b: bool) {
    let mut x = 1;
    if a {
        x = (156 / (156 % 156));
    }
    if b {
        x = func(156, 3);
    }
    unsafe { verify_assert(x != 1); }
}

fn func(a: u8, b: u8) -> u8 {
    (a / (a % b))
}

unconstrained fn verify_assert(b: bool) { assert(b); }
