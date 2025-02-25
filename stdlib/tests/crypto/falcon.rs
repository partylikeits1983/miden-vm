use std::vec;

use assembly::{utils::Serializable, Assembler};
use miden_air::{Felt, ProvingOptions, RowIndex};
use miden_stdlib::StdLibrary;
use processor::{
    crypto::RpoRandomCoin, AdviceInputs, DefaultHost, Digest, ExecutionError, MemAdviceProvider,
    Program, ProgramInfo, StackInputs,
};
use rand::{thread_rng, Rng};
use test_utils::{
    crypto::{
        rpo_falcon512::{Polynomial, SecretKey},
        MerkleStore, Rpo256,
    },
    expect_exec_error_matches,
    proptest::proptest,
    rand::rand_vector,
    Word,
};
use vm_core::StarkField;

/// Modulus used for rpo falcon 512.
const M: u64 = 12289;
const Q: u64 = (M - 1) / 2;
const N: usize = 512;
const J: u64 = (N * M as usize * M as usize) as u64;

const PROBABILISTIC_PRODUCT_SOURCE: &str = "
    use.std::crypto::dsa::rpo_falcon512

    begin
        #=> [PK, ...]
        push.0
        #=> [h_ptr, PK, ...]

        exec.rpo_falcon512::load_h_s2_and_product
        #=> [...]
    end
    ";

#[test]
fn test_falcon512_norm_sq() {
    let source = "
    use.std::crypto::dsa::rpo_falcon512

    begin
        exec.rpo_falcon512::norm_sq
    end
    ";

    // normalize(e) = e^2 - phi * (2*M*e - M^2) where phi := (e > (M - 1)/2)
    let upper = rand::thread_rng().gen_range(Q + 1..M);
    let test_upper = build_test!(source, &[upper]);
    test_upper.expect_stack(&[(M - upper) * (M - upper)]);

    let lower = rand::thread_rng().gen_range(0..=Q);
    let test_lower = build_test!(source, &[lower]);
    test_lower.expect_stack(&[lower * lower])
}

#[test]
fn test_falcon512_diff_mod_m() {
    let source = "
    use.std::crypto::dsa::rpo_falcon512

    begin
        exec.rpo_falcon512::diff_mod_M
    end
    ";
    let v = Felt::MODULUS - 1;
    let (v_lo, v_hi) = (v as u32, v >> 32);

    // test largest possible value given v
    let w = J - 1;
    let u = 0;

    let test1 = build_test!(source, &[v_lo as u64, v_hi, w + J, u]);

    // Calculating (v - (u + (- w % M) % M) % M) should be the same as (v + w + J - u) % M.
    let expanded_answer = (v as i128
        - ((u as i64 + -(w as i64).rem_euclid(M as i64)).rem_euclid(M as i64) as i128))
        .rem_euclid(M as i128);
    let simplified_answer = (v as i128 + w as i128 + J as i128 - u as i128).rem_euclid(M as i128);
    assert_eq!(expanded_answer, simplified_answer);

    test1.expect_stack(&[simplified_answer as u64]);

    // test smallest possible value given v
    let w = 0;
    let u = J - 1;

    let test2 = build_test!(source, &[v_lo as u64, v_hi, w + J, u]);

    // Calculating (v - (u + (- w % M) % M) % M) should be the same as (v + w + J - u) % M.
    let expanded_answer = (v as i128
        - ((u as i64 + -(w as i64).rem_euclid(M as i64)).rem_euclid(M as i64) as i128))
        .rem_euclid(M as i128);
    let simplified_answer = (v as i128 + w as i128 + J as i128 - u as i128).rem_euclid(M as i128);
    assert_eq!(expanded_answer, simplified_answer);

    test2.expect_stack(&[simplified_answer as u64]);
}

proptest! {
    #[test]
    fn diff_mod_m_proptest(v in 0..Felt::MODULUS, w in 0..J, u in 0..J) {

          let source = "
    use.std::crypto::dsa::rpo_falcon512

    begin
        exec.rpo_falcon512::diff_mod_M
    end
    ";

    let (v_lo, v_hi) = (v as u32, v >> 32);

    let test1 = build_test!(source, &[v_lo as u64, v_hi, w + J, u]);

    // Calculating (v - (u + (- w % M) % M) % M) should be the same as (v + w + J - u) % M.
    let expanded_answer = (v as i128
        - ((u as i64 + -(w as i64).rem_euclid(M as i64)).rem_euclid(M as i64) as i128))
    .rem_euclid(M as i128);
    let simplified_answer = (v as i128 + w as i128 + J as i128 - u as i128).rem_euclid(M as i128);
    assert_eq!(expanded_answer, simplified_answer);

    test1.prop_expect_stack(&[simplified_answer as u64])?;
    }

}

#[test]
fn test_falcon512_probabilistic_product() {
    // create two random polynomials and generate the input operand stack and advice stack to
    // the probabilistic product test procedure
    let h: Polynomial<Felt> = Polynomial::new(random_coefficients());
    let s2: Polynomial<Felt> = Polynomial::new(random_coefficients());
    let (operand_stack, advice_stack): (Vec<u64>, Vec<u64>) =
        generate_data_probabilistic_product_test(h, s2, false);

    let test = build_test!(PROBABILISTIC_PRODUCT_SOURCE, &operand_stack, &advice_stack);
    let expected_stack = &[];
    test.expect_stack(expected_stack);
}

#[test]
fn test_falcon512_probabilistic_product_failure() {
    // create two random polynomials and generate the input operand stack and advice stack to
    // the probabilistic product test procedure
    let h: Polynomial<Felt> = Polynomial::new(random_coefficients());
    let s2: Polynomial<Felt> = Polynomial::new(random_coefficients());
    let (operand_stack, advice_stack): (Vec<u64>, Vec<u64>) =
        generate_data_probabilistic_product_test(h, s2, true);

    let test = build_test!(PROBABILISTIC_PRODUCT_SOURCE, &operand_stack, &advice_stack);

    expect_exec_error_matches!(
        test,
        ExecutionError::FailedAssertion{ clk, err_code, err_msg }
        if clk == RowIndex::from(3182) && err_code == 0 && err_msg.is_none()
    );
}

#[test]
fn falcon_execution() {
    let seed = Word::default();
    let mut rng = RpoRandomCoin::new(seed);
    let sk = SecretKey::with_rng(&mut rng);
    let message = rand_vector::<Felt>(4).try_into().unwrap();
    let (source, op_stack, adv_stack, store, advice_map) = generate_test(sk, message);

    let test = build_test!(source, &op_stack, &adv_stack, store, advice_map.into_iter());
    test.expect_stack(&[])
}

#[test]
#[ignore]
fn falcon_prove_verify() {
    let sk = SecretKey::new();
    let message = rand_vector::<Felt>(4).try_into().unwrap();
    let (source, op_stack, _, _, advice_map) = generate_test(sk, message);

    let program: Program = Assembler::default()
        .with_library(StdLibrary::default())
        .expect("failed to load stdlib")
        .assemble_program(source)
        .expect("failed to compile test source");

    let stack_inputs = StackInputs::try_from_ints(op_stack).expect("failed to create stack inputs");
    let advice_inputs = AdviceInputs::default().with_map(advice_map);
    let advice_provider = MemAdviceProvider::from(advice_inputs);
    let mut host = DefaultHost::new(advice_provider);

    let options = ProvingOptions::with_96_bit_security(false);
    let (stack_outputs, proof) =
        test_utils::prove(&program, stack_inputs.clone(), &mut host, options)
            .expect("failed to generate proof");

    let program_info = ProgramInfo::from(program);
    let result = test_utils::verify(program_info, stack_inputs, stack_outputs, proof);

    assert!(result.is_ok(), "error: {result:?}");
}

#[allow(clippy::type_complexity)]
fn generate_test(
    sk: SecretKey,
    message: Word,
) -> (&'static str, Vec<u64>, Vec<u64>, MerkleStore, Vec<(Digest, Vec<Felt>)>) {
    let source = "
    use.std::crypto::dsa::rpo_falcon512

    begin
        exec.rpo_falcon512::verify
    end
    ";

    let pk: Word = sk.public_key().into();
    let pk: Digest = pk.into();
    let sk_bytes = sk.to_bytes();

    let to_adv_map = sk_bytes.iter().map(|a| Felt::new(*a as u64)).collect::<Vec<Felt>>();

    let advice_map: Vec<(Digest, Vec<Felt>)> = vec![(pk, to_adv_map)];

    let mut op_stack = vec![];
    let message = message.into_iter().map(|a| a.as_int()).collect::<Vec<u64>>();
    op_stack.extend_from_slice(&message);
    op_stack.extend_from_slice(&pk.as_elements().iter().map(|a| a.as_int()).collect::<Vec<u64>>());
    let adv_stack = vec![];
    let store = MerkleStore::new();

    (source, op_stack, adv_stack, store, advice_map)
}

// HELPER FUNCTIONS
// ================================================================================================

/// Creates random coefficients of a polynomial in the range (0..M).
fn random_coefficients() -> Vec<Felt> {
    let mut res = Vec::new();
    for _i in 0..N {
        res.push(Felt::new(thread_rng().gen_range(0..M)))
    }
    res
}

/// Multiplies two polynomials over Z_p\[x\] without reducing modulo p.
///
/// Given that the degrees of the input polynomials are less than 512 and their coefficients are
/// less than the modulus M = 12289, the resulting product polynomial is guaranteed to have
/// coefficients less than the Miden prime.
///
/// Note that this multiplication is not over Z_p\[x\]/(phi).
fn mul_modulo_p(a: Polynomial<Felt>, b: Polynomial<Felt>) -> [u64; 1024] {
    let mut c = [0; 2 * N];
    for i in 0..N {
        for j in 0..N {
            c[i + j] += a.coefficients[i].as_int() * b.coefficients[j].as_int();
        }
    }
    c
}

/// Returns the coefficients of a polynomial.
fn to_elements(poly: Polynomial<Felt>) -> Vec<Felt> {
    poly.coefficients.to_vec()
}

/// Generates the data needed to execute the probabilistic product test.
fn generate_data_probabilistic_product_test(
    h: Polynomial<Felt>,
    s2: Polynomial<Felt>,
    test_failure: bool,
) -> (Vec<u64>, Vec<u64>) {
    let pi = mul_modulo_p(h.clone(), s2.clone());

    // lay the polynomials in order h then s2 then pi = h * s2
    let mut polynomials = if test_failure {
        to_elements(Polynomial::new(random_coefficients()))
    } else {
        to_elements(h.clone())
    };
    polynomials.extend(to_elements(s2.clone()));
    polynomials.extend(pi.iter().map(|a| Felt::new(*a)));

    // get the challenge point and push it to the advice stack
    let digest_polynomials = Rpo256::hash_elements(&polynomials);
    let challenge = (digest_polynomials[0], digest_polynomials[1]);
    let mut advice_stack = vec![challenge.0.as_int(), challenge.1.as_int()];

    // push the polynomials to the advice stack
    let polynomials: Vec<u64> = polynomials.iter().map(|&e| e.into()).collect();
    advice_stack.extend_from_slice(&polynomials);

    // compute hash of h and place it on the stack.
    let binding = Rpo256::hash_elements(&to_elements(h.clone()));
    let h_hash = binding.as_elements();
    let h_hash_copy: Vec<u64> = h_hash.iter().map(|felt| (*felt).into()).collect();
    let operand_stack = vec![h_hash_copy[0], h_hash_copy[1], h_hash_copy[2], h_hash_copy[3]];

    (operand_stack, advice_stack)
}
