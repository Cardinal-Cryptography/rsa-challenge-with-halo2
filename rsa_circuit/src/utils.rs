//! Helpers for working with the RSA circuit.

use halo2_proofs::{
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::kzg::commitment::ParamsKZG,
};
use rand::rngs::OsRng;

use crate::RsaChallenge;

/// Initial setup artifacts including trusted setup, proving key and verifying key.
pub struct Setup {
    /// Logarithm of the maximum number of rows in the PLONK table.
    pub k: u32,
    /// Proving key.
    pub pk: ProvingKey<G1Affine>,
    /// Verifying key.
    pub vk: VerifyingKey<G1Affine>,
    /// Trusted setup.
    pub params: ParamsKZG<Bn256>,
}

/// Run the initial setup phase (for SRS) and circuit processing (for keys).
pub fn generate_setup(k: u32) -> Setup {
    let circuit = RsaChallenge::default();
    let params = ParamsKZG::<Bn256>::setup(k, OsRng);
    let vk = keygen_vk(&params, &circuit).expect("vk generation should not fail");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk generation should not fail");
    Setup { k, pk, vk, params }
}

/// Convert the public input from human-readable form to the scalar array.
pub fn prepare_public_input(n: u128, account: [u8; 32]) -> [Fr; 3] {
    [
        Fr::from_u128(n),
        Fr::from_u128(u128::from_le_bytes(account[..16].try_into().unwrap())),
        Fr::from_u128(u128::from_le_bytes(account[16..].try_into().unwrap())),
    ]
}
