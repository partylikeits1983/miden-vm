mod verifier_recursive;
use verifier_recursive::{generate_advice_inputs, VerifierData};

use assembly::Assembler;
use miden_air::{FieldExtension, HashFunction, PublicInputs};
use processor::DefaultHost;
use test_utils::{
    prove, AdviceInputs, MemAdviceProvider, ProgramInfo, ProvingOptions, StackInputs, VerifierError,
};

// Note: Changes to MidenVM may cause this test to fail when some of the assumptions documented
// in `stdlib/asm/crypto/stark/verifier.masm` are violated.
#[test]
#[ignore]
fn stark_verifier_e2f4() {
    // An example MASM program to be verified inside Miden VM.
    // Note that output stack-overflow is not yet supported because of the way we handle public
    // inputs in the STARK verifier is not yet general enough. Thus the output stack should be
    // of size exactly 16.
    let example_source = "begin
            repeat.32
                swap dup.1 add
            end
        end";
    let mut stack_inputs = vec![0_u64; 16];
    stack_inputs[15] = 0;
    stack_inputs[14] = 1;

    let VerifierData {
        initial_stack,
        tape,
        store,
        advice_map,
    } = generate_recursive_verifier_data(example_source, stack_inputs).unwrap();

    // Verify inside Miden VM
    let source = "
        use.std::crypto::stark::verifier
        begin
            exec.verifier::verify
        end
        ";

    let test = build_test!(source, &initial_stack, &tape, store, advice_map);

    test.expect_stack(&[]);
}

// Helper function for recursive verification
pub fn generate_recursive_verifier_data(
    source: &str,
    stack_inputs: Vec<u64>,
) -> Result<VerifierData, VerifierError> {
    let program = Assembler::default().assemble(source).unwrap();
    let stack_inputs = StackInputs::try_from_ints(stack_inputs).unwrap();
    let advice_inputs = AdviceInputs::default();
    let advice_provider = MemAdviceProvider::from(advice_inputs);
    let host = DefaultHost::new(advice_provider);

    let options =
        ProvingOptions::new(43, 8, 12, FieldExtension::Quadratic, 4, 7, HashFunction::Rpo256);

    let (stack_outputs, proof) = prove(&program, stack_inputs.clone(), host, options).unwrap();

    let program_info = ProgramInfo::from(program);

    // build public inputs and generate the advice data needed for recursive proof verification
    let pub_inputs = PublicInputs::new(program_info, stack_inputs, stack_outputs);
    let (_, proof) = proof.into_parts();
    Ok(generate_advice_inputs(proof, pub_inputs).unwrap())
}
