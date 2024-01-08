use halo2_proofs::{
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{create_proof, keygen_pk, keygen_vk, verify_proof, Error, VerifyingKey},
    poly::kzg::{
        commitment::ParamsKZG,
        multiopen::{ProverGWC, VerifierGWC},
        strategy::SingleStrategy,
    },
    transcript::{
        Blake2bRead, Blake2bWrite, Challenge255, TranscriptReadBuffer, TranscriptWriterBuffer,
    },
};
use rand::rngs::OsRng;

use crate::{prepare_public_input, RsaChallenge};

const CIRCUIT_MAX_K: u32 = 5;
const ACCOUNT: [u8; 32] = [0u8; 32];
const FAKE_ACCOUNT: [u8; 32] = [1u8; 32];

struct Setup {
    proof: Vec<u8>,
    instances: [Fr; 3],
    vk: VerifyingKey<G1Affine>,
    params: ParamsKZG<Bn256>,
}

fn setup(p: u128, q: u128, account: [u8; 32]) -> Setup {
    let n = p * q;

    let circuit = RsaChallenge {
        p: Fr::from_u128(p),
        q: Fr::from_u128(q),
    };
    let instances = prepare_public_input(n, account);

    let params = ParamsKZG::<Bn256>::setup(CIRCUIT_MAX_K, OsRng);
    let vk = keygen_vk(&params, &circuit).expect("vk should not fail");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk should not fail");

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

    Setup {
        proof,
        instances,
        vk,
        params,
    }
}

fn verify(setup: Setup) -> Result<(), Error> {
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
    let fake_setup = Setup {
        instances: prepare_public_input(41 * 43, FAKE_ACCOUNT),
        ..true_setup
    };
    assert!(verify(fake_setup).is_err());
}

#[test]
fn does_not_accept_incorrect_proof() {
    let true_setup = setup(41, 43, ACCOUNT);
    let alternative_setup = setup(11, 13, ACCOUNT);
    let fake_setup = Setup {
        proof: alternative_setup.proof,
        ..true_setup
    };
    assert!(verify(fake_setup).is_err());
}
