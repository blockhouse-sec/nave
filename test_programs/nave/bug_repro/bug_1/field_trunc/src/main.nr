//! Bug: https://github.com/noir-lang/noir/issues/7962
//! Fix: https://github.com/noir-lang/noir/pull/8010
//!
//! Root cause: incorrect truncation of field elements.
//!
//! v8 = v3 - 1
//! result = truncate v8 to 1 bit, max_bit_size: 7
//!
//! The problem is not specific to 8-bit truncation or bitwise AND.
//! It comes from the truncate operation not updating the valid bit
//! range after arithmetic changes to the operand.
//!
//! When a value is modified (e.g. subtraction), its range can change.
//! However, `max_bit_size` was still set to 7, which is incorrect.
//!
//! This wrong constraint makes the solver return UNSAT for most values
//! (e.g. 25), even though a valid solution exists. The small bit range
//! creates a contradiction.
//! When corrected to 8 bits, the solver finds valid assignments (SAT).
//!
//! The incorrect 'Verified' result from the verifier helps expose this bug.
//!
//! The `field_trunc` directory contains SMT outputs for both the buggy
//! (7-bit) and fixed (8-bit) versions.

fn main(v3: u8) -> pub u8 {
    let v8 = v3 - 1;
    assert(v3 == 127);
    
    let result = v8 & 1;
    unsafe { verify_assert(result == 25); } // Verified??

    result
}

unconstrained fn verify_assert(b: bool) { assert(b); }