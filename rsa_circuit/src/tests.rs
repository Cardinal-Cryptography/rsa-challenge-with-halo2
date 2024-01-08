use halo2_proofs::{
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{create_proof, verify_proof, Error, VerifyingKey},
    poly::kzg::{
        commitment::ParamsKZG,
        multiopen::{ProverGWC, VerifierGWC},
        strategy::SingleStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
    SerdeFormat,
};
use rand::rngs::OsRng;

use crate::{
    utils::{generate_setup, prepare_public_input, Setup},
    RsaChallenge,
};

const CIRCUIT_MAX_K: u32 = 5;
const ACCOUNT: [u8; 32] = [0u8; 32];
const FAKE_ACCOUNT: [u8; 32] = [1u8; 32];

struct TestSetup {
    proof: Vec<u8>,
    instances: [Fr; 3],
    vk: VerifyingKey<G1Affine>,
    params: ParamsKZG<Bn256>,
}

fn setup(p: u128, q: u128, account: [u8; 32]) -> TestSetup {
    let Setup { pk, vk, params, .. } = generate_setup(CIRCUIT_MAX_K);

    let n = p * q;
    let circuit = RsaChallenge {
        p: Some(Fr::from_u128(p)),
        q: Some(Fr::from_u128(q)),
    };
    let instances = prepare_public_input(n, account);

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<_, ProverGWC<'_, Bn256>, _, _, _, _>(
        &params,
        &pk,
        &[circuit],
        &[&[&instances]],
        OsRng,
        &mut transcript,
    )
    .expect("prover should not fail");
    let proof = transcript.finalize();

    TestSetup {
        proof,
        instances,
        vk,
        params,
    }
}

fn verify(setup: TestSetup) -> Result<(), Error> {
    verify_proof::<_, VerifierGWC<_>, _, _, _>(
        &setup.params,
        &setup.vk,
        SingleStrategy::new(&setup.params),
        &[&[&setup.instances]],
        &mut Blake2bRead::init(&setup.proof[..]),
    )
}

#[test]
fn accepts_correct_proof() {
    assert!(verify(setup(41, 43, ACCOUNT)).is_ok());
}

#[test]
fn works_with_big_numbers() {
    assert!(verify(setup(7413101572609314289, 6786072055295288333, ACCOUNT)).is_ok());
}

#[test]
fn does_not_accept_fake_account() {
    let true_setup = setup(41, 43, ACCOUNT);
    let fake_setup = TestSetup {
        instances: prepare_public_input(41 * 43, FAKE_ACCOUNT),
        ..true_setup
    };
    assert!(verify(fake_setup).is_err());
}

#[test]
fn does_not_accept_incorrect_proof() {
    let true_setup = setup(41, 43, ACCOUNT);
    let alternative_setup = setup(11, 13, ACCOUNT);
    let fake_setup = TestSetup {
        proof: alternative_setup.proof,
        ..true_setup
    };
    assert!(verify(fake_setup).is_err());
}

#[test]
fn setup_serialization_works() {
    let setup = generate_setup(CIRCUIT_MAX_K);
    let serialized = setup.clone().to_bytes();
    let deserialized = Setup::from_bytes(&mut serialized.as_slice());

    assert_eq!(setup.k, deserialized.k);
    assert_eq!(setup.params.s_g2(), deserialized.params.s_g2());
    assert_eq!(
        setup.pk.to_bytes(SerdeFormat::RawBytesUnchecked),
        deserialized.pk.to_bytes(SerdeFormat::RawBytesUnchecked)
    );
}
