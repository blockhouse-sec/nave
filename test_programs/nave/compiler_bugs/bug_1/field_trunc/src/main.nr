//! Bug: https://github.com/noir-lang/noir/issues/7962
//! Fix: https://github.com/noir-lang/noir/pull/8010
//!
//! Function argument `v3` is truncated after subtracting one from it; v8
//! is the value produced by the subtraction. The issue arises during
//! ACIR compilation where the compiler attempts to decompose v8 + 256
//! into 2x + y, where x is a 7-bit unsigned integer and y is a bit.
//! The value 256 is a compiler trick used to avoid underflow. However,
//! this trick makes decomposition into 2x + y impossible because v8 is
//! an 8-bit unsigned integer and the value 256 + z for any z >= 0 needs
//! 9 bits to be represented. 8 bits represent numbers in the range
//! [0, 255].
//!
//! The `verify_assert(false)` statement acts as a reachability check.
//! The solver returns 'Verified' meaning no input can make the constraints
//! satisfiable.

fn main(v3: u8) -> pub u8 {
    let v8 = v3 - 1;
    let result = v8 & 1;
    unsafe {
        verify_assert(false);
    }
    result
}

unconstrained fn verify_assert(b: bool) { assert(b); }