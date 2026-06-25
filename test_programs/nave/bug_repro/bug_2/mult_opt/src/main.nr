//! Wrapping multiplication of 16-bit unsigned integers using 8-bit limbs, with a bug
//! in the high part of the multiplication to test how the SMT solver handles incorrect 
//! arithmetic in this context.
//! 
//! Fix introduced for 128 bits in PR: https://github.com/noir-lang/noir/pull/5024
//! 
//! We use 8-bit limbs instead of 64-bit limbs to amplify the bug's effect, making it more
//! likely to produce incorrect results and easier to verify the buggy behavior.
//! With 64-bit limbs, the larger value range would reduce the chance of visible errors,
//! and the increased complexity would make solver computations more challenging.

fn wrapping_mul8(x: u16, y: u16) -> u16 {
    let (x_lo, x_hi) = (x as u8, (x / 256) as u8);
    let (y_lo, y_hi) = (y as u8, (y / 256) as u8);
    let low = (x_lo as u16 * y_lo as u16);
    let lo = low as u8 as u16;
    let carry = (low - lo) / 256;
    let high = 
    // if 254 as u32 > 196  {
    // original condition that was wrong and caused the buggy computation to be used for all inputs
        (x_lo + x_hi) as u16 * (y_lo + y_hi) as u16 - low + carry;
    // } else {
        // The correct computation for the high part of the multiplication is the following
        // However, we intentionally use the buggy version above 
        // (always executed since the condition is commented out) to test
        // how the SMT solver handles incorrect arithmetic in this context.

    //     x_lo as Field * y_hi as Field + x_hi as Field * y_lo as Field + carry
    // };
    let hi = high as u8 as u16;
    (lo + 256 * hi)
}

fn main() -> pub u16 {
    unsafe { verify_assert(wrapping_mul8(2, 2) == 4); } // Verified
    unsafe { verify_assert(wrapping_mul8(254, 254) == (254 * 254)); } // Verified

    let result = wrapping_mul8(256, 256);
    unsafe { verify_assert(result == 0); } // Falsified
    result
}

unconstrained fn verify_assert(b: bool) { assert(b); }