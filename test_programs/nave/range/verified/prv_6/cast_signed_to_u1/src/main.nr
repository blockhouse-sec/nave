fn main(
    zero: i32,
    one: i32,
    two: i32,
    three: i32,
    minus_one: i32,
    minus_two: i32,
    minus_three: i32,
) {
    unsafe { verify_assert(zero as bool == false); }
    unsafe { verify_assert(one as bool == true); }
    unsafe { verify_assert(two as bool == false); }
    unsafe { verify_assert(three as bool == true); }
    unsafe { verify_assert(minus_one as bool == true); }
    unsafe { verify_assert(minus_two as bool == false); }
    unsafe { verify_assert(minus_three as bool == true); }

    let zero: i32 = 0;
    let one: i32 = 1;
    let two: i32 = 2;
    let three: i32 = 3;
    let minus_one: i32 = -1;
    let minus_two: i32 = -2;
    let minus_three: i32 = -3;
    unsafe { verify_assert(zero as bool == false); }
    unsafe { verify_assert(one as bool == true); }
    unsafe { verify_assert(two as bool == false); }
    unsafe { verify_assert(three as bool == true); }
    unsafe { verify_assert(minus_one as bool == true); }
    unsafe { verify_assert(minus_two as bool == false); }
    unsafe { verify_assert(minus_three as bool == true); };
}
unconstrained fn verify_assert(b: bool) { assert(false); }
