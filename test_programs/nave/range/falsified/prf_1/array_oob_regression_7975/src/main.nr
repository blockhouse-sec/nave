fn main(b: [[bool; 1]; 1]) -> pub [[bool; 1]; 1] {
    unsafe { verify_assert(b[0][0] == 1); }
    if (b[0][0] == b[0][0]) {
        b
    } else {
        b
    }
}
unconstrained fn verify_assert(b: bool) { assert(false); }
