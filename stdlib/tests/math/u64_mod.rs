use core::cmp;
use processor::ExecutionError;
use test_utils::{
    expect_exec_error, proptest::prelude::*, rand::rand_value, Felt, U32_BOUND, ZERO,
};

// ADDITION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_add() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_add(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_add
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn overflowing_add() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_add
        end";

    let a = rand_value::<u64>() as u32 as u64;
    let b = rand_value::<u64>() as u32 as u64;
    let (c, _) = a.overflowing_add(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[0, c1, c0]);

    let a = u64::MAX;
    let b = rand_value::<u64>();
    let (c, _) = a.overflowing_add(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[1, c1, c0]);
}

// SUBTRACTION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_sub() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_sub(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_sub
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn overflowing_sub() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let (c, flag) = a.overflowing_sub(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::overflowing_sub
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[flag as u64, c1, c0]);

    let base = rand_value::<u64>() as u32 as u64;
    let diff = rand_value::<u64>() as u32 as u64;

    let a = base;
    let b = base + diff;
    let (c, _) = a.overflowing_sub(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[1, c1, c0]);

    let base = rand_value::<u64>() as u32 as u64;
    let diff = rand_value::<u64>() as u32 as u64;

    let a = base + diff;
    let b = base;
    let (c, _) = a.overflowing_sub(b);

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[0, c1, c0]);
}

// MULTIPLICATION
// ------------------------------------------------------------------------------------------------

#[test]
fn wrapping_mul() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a.wrapping_mul(b);

    let source = "
        use.std::math::u64
        begin
            exec.u64::wrapping_mul
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn overflowing_mul() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::overflowing_mul
    end";

    let a = u64::MAX as u128;
    let b = u64::MAX as u128;
    let c = a.wrapping_mul(b);

    let a = u64::MAX;
    let b = u64::MAX;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c3, c2, c1, c0) = split_u128(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c3, c2, c1, c0]);

    let a = rand_value::<u64>() as u128;
    let b = rand_value::<u64>() as u128;
    let c = a.wrapping_mul(b);

    let a = a as u64;
    let b = b as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c3, c2, c1, c0) = split_u128(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c3, c2, c1, c0]);
}

// COMPARISONS
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_lt() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::lt
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);
}

#[test]
fn unchecked_lte() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::lte
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a <= b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_gt() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::gt
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);
}

#[test]
fn unchecked_gte() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::gte
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a >= b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_min() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::min
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0, 0]);

    // a = 1, b = 2
    build_test!(source, &[1, 0, 2, 0]).expect_stack(&[0, 1]);

    // a = 3, b = 2
    build_test!(source, &[3, 0, 2, 0]).expect_stack(&[0, 2]);
}

#[test]
fn unchecked_max() {
    // test a few manual cases; randomized tests are done using proptest
    let source = "
        use.std::math::u64
        begin
            exec.u64::max
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0, 0]);

    // a = 1, b = 2
    build_test!(source, &[1, 0, 2, 0]).expect_stack(&[0, 2]);

    // a = 3, b = 2
    build_test!(source, &[3, 0, 2, 0]).expect_stack(&[0, 3]);
}

#[test]
fn unchecked_eq() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::eq
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[1]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[0]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a == b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_neq() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::neq
        end";

    // a = 0, b = 0
    build_test!(source, &[0, 0, 0, 0]).expect_stack(&[0]);

    // a = 0, b = 1
    build_test!(source, &[0, 0, 1, 0]).expect_stack(&[1]);

    // a = 1, b = 0
    build_test!(source, &[1, 0, 0, 0]).expect_stack(&[1]);

    // randomized test
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = (a != b) as u64;

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    build_test!(source, &[a0, a1, b0, b1]).expect_stack(&[c]);
}

#[test]
fn unchecked_eqz() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::eqz
        end";

    // a = 0
    build_test!(source, &[0, 0]).expect_stack(&[1]);

    // a = 1
    build_test!(source, &[1, 0]).expect_stack(&[0]);

    // randomized test
    let a: u64 = rand_value();
    let c = (a == 0) as u64;

    let (a1, a0) = split_u64(a);
    build_test!(source, &[a0, a1]).expect_stack(&[c]);
}

// DIVISION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_div() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a / b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::div
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);

    let d = a / b0;
    let (d1, d0) = split_u64(d);

    let test = build_test!(source, &[a0, a1, b0, 0]);
    test.expect_stack(&[d1, d0]);
}

// MODULO OPERATION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_mod() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a % b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::mod
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);

    let d = a % b0;
    let (d1, d0) = split_u64(d);

    let test = build_test!(source, &[a0, a1, b0, 0]);
    test.expect_stack(&[d1, d0]);
}

// DIVMOD OPERATION
// ------------------------------------------------------------------------------------------------

#[test]
fn unchecked_divmod() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let q = a / b;
    let r = a % b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::divmod
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (q1, q0) = split_u64(q);
    let (r1, r0) = split_u64(r);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[r1, r0, q1, q0]);
}

// BITWISE OPERATIONS
// ------------------------------------------------------------------------------------------------

#[test]
fn checked_and() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a & b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::and
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_and_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = U32_BOUND;
    let b1: u64 = U32_BOUND;

    let source = "
        use.std::math::u64
        begin
            exec.u64::and
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    expect_exec_error!(test, ExecutionError::NotU32Value(Felt::new(b0), ZERO));
}

#[test]
fn checked_or() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a | b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::or
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_or_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = U32_BOUND;
    let b1: u64 = U32_BOUND;

    let source = "
        use.std::math::u64
        begin
            exec.u64::or
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    expect_exec_error!(test, ExecutionError::NotU32Value(Felt::new(b0), ZERO));
}

#[test]
fn checked_xor() {
    let a: u64 = rand_value();
    let b: u64 = rand_value();
    let c = a ^ b;

    let source = "
        use.std::math::u64
        begin
            exec.u64::xor
        end";

    let (a1, a0) = split_u64(a);
    let (b1, b0) = split_u64(b);
    let (c1, c0) = split_u64(c);

    let test = build_test!(source, &[a0, a1, b0, b1]);
    test.expect_stack(&[c1, c0]);
}

#[test]
fn checked_xor_fail() {
    let a0: u64 = rand_value();
    let b0: u64 = rand_value();

    let a1: u64 = U32_BOUND;
    let b1: u64 = U32_BOUND;

    let source = "
        use.std::math::u64
        begin
            exec.u64::xor
        end";

    let test = build_test!(source, &[a0, a1, b0, b1]);
    expect_exec_error!(test, ExecutionError::NotU32Value(Felt::new(b0), ZERO));
}

#[test]
#[ignore]
fn unchecked_shl() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::shl
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.wrapping_shl(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn unchecked_shr() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::shr
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // simple right shift
    build_test!(source, &[5, 1, 1, 1]).expect_stack(&[0, 2_u64.pow(31), 5]);

    // simple right shift
    build_test!(source, &[5, 3, 3, 1]).expect_stack(&[1, 2_u64.pow(31) + 1, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 4294967296 by 2
    let a = 4294967296;
    let (a1, a0) = split_u64(a);
    let b: u32 = 2;
    let c = a.wrapping_shr(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn unchecked_rotl() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::rotl
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.rotate_left(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn unchecked_rotr() {
    let source = "
        use.std::math::u64
        begin
            exec.u64::rotr
        end";

    // shift by 0
    let a: u64 = rand_value();
    let (a1, a0) = split_u64(a);
    let b: u32 = 0;

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[a1, a0, 5]);

    // shift by 31 (max lower limb of b)
    let b: u32 = 31;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 32 (min for upper limb of b)
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 32;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift by 33
    let a = 1_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 33;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);

    // shift 64 by 58
    let a = 64_u64;
    let (a1, a0) = split_u64(a);
    let b: u32 = 58;
    let c = a.rotate_right(b);
    let (c1, c0) = split_u64(c);

    build_test!(source, &[5, a0, a1, b as u64]).expect_stack(&[c1, c0, 5]);
}

#[test]
fn clz() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::clz
    end";

    build_test!(source, &[0, 0]).expect_stack(&[64]);
    build_test!(source, &[492665065, 0]).expect_stack(&[35]);
    build_test!(source, &[3941320520, 0]).expect_stack(&[32]);
    build_test!(source, &[3941320520, 492665065]).expect_stack(&[3]);
    build_test!(source, &[492665065, 492665065]).expect_stack(&[3]);
}

#[test]
fn ctz() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::ctz
    end";

    build_test!(source, &[0, 0]).expect_stack(&[64]);
    build_test!(source, &[0, 3668265216]).expect_stack(&[40]);
    build_test!(source, &[0, 3668265217]).expect_stack(&[32]);
    build_test!(source, &[3668265216, 3668265217]).expect_stack(&[8]);
    build_test!(source, &[3668265216, 3668265216]).expect_stack(&[8]);
}

#[test]
fn clo() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::clo
    end";

    build_test!(source, &[4294967295, 4294967295]).expect_stack(&[64]);
    build_test!(source, &[4278190080, 4294967295]).expect_stack(&[40]);
    build_test!(source, &[0, 4294967295]).expect_stack(&[32]);
    build_test!(source, &[0, 4278190080]).expect_stack(&[8]);
    build_test!(source, &[4278190080, 4278190080]).expect_stack(&[8]);
}

#[test]
fn cto() {
    let source = "
    use.std::math::u64
    begin
        exec.u64::cto
    end";

    build_test!(source, &[4294967295, 4294967295]).expect_stack(&[64]);
    build_test!(source, &[4294967295, 255]).expect_stack(&[40]);
    build_test!(source, &[4294967295, 0]).expect_stack(&[32]);
    build_test!(source, &[255, 0]).expect_stack(&[8]);
    build_test!(source, &[255, 255]).expect_stack(&[8]);
}

// RANDOMIZED TESTS
// ================================================================================================

proptest! {
    #[test]
    fn unchecked_lt_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = (a < b) as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::lt
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn unchecked_gt_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = (a > b) as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::gt
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn unchecked_min_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = cmp::min(a, b);
        let (c1, c0) = split_u64(c);
        let source = "
            use.std::math::u64
            begin
                exec.u64::min
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_max_proptest(a in any::<u64>(), b in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let c = cmp::max(a, b);
        let (c1, c0) = split_u64(c);
        let source = "
            use.std::math::u64
            begin
                exec.u64::max
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_div_proptest(a in any::<u64>(), b in any::<u64>()) {

        let c = a / b;

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let (c1, c0) = split_u64(c);

        let source = "
            use.std::math::u64
            begin
                exec.u64::div
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn unchecked_mod_proptest(a in any::<u64>(), b in any::<u64>()) {

        let c = a % b;

        let (a1, a0) = split_u64(a);
        let (b1, b0) = split_u64(b);
        let (c1, c0) = split_u64(c);

        let source = "
            use.std::math::u64
            begin
                exec.u64::mod
            end";

        build_test!(source, &[a0, a1, b0, b1]).prop_expect_stack(&[c1, c0])?;
    }

    #[test]
    fn shl_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.wrapping_shl(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::shl
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn shr_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.wrapping_shr(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::shr
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn rotl_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.rotate_left(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::rotl
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn rotr_proptest(a in any::<u64>(), b in 0_u32..64) {

        let c = a.rotate_right(b);

        let (a1, a0) = split_u64(a);
        let (c1, c0) = split_u64(c);

        let source = "
        use.std::math::u64
        begin
            exec.u64::rotr
        end";

        build_test!(source, &[5, a0, a1, b as u64]).prop_expect_stack(&[c1, c0, 5])?;
    }

    #[test]
    fn clz_proptest(a in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let c = a.leading_zeros() as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::clz
            end";

        build_test!(source, &[a0, a1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn ctz_proptest(a in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let c = a.trailing_zeros() as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::ctz
            end";

        build_test!(source, &[a0, a1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn clo_proptest(a in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let c = a.leading_ones() as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::clo
            end";

        build_test!(source, &[a0, a1]).prop_expect_stack(&[c])?;
    }

    #[test]
    fn cto_proptest(a in any::<u64>()) {

        let (a1, a0) = split_u64(a);
        let c = a.trailing_ones() as u64;

        let source = "
            use.std::math::u64
            begin
                exec.u64::cto
            end";

        build_test!(source, &[a0, a1]).prop_expect_stack(&[c])?;
    }
}

// HELPER FUNCTIONS
// ================================================================================================

/// Split the provided u64 value into 32 high and low bits.
fn split_u64(value: u64) -> (u64, u64) {
    (value >> 32, value as u32 as u64)
}

fn split_u128(value: u128) -> (u64, u64, u64, u64) {
    (
        (value >> 96) as u64,
        (value >> 64) as u32 as u64,
        (value >> 32) as u32 as u64,
        value as u32 as u64,
    )
}
