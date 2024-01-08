//! Helpers for working with the RSA circuit.

use halo2_proofs::{
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
    standard_plonk::StandardPlonk,
    SerdeFormat,
};
use rand::rngs::OsRng;

use crate::RsaChallenge;

const SERDE_FORMAT: SerdeFormat = SerdeFormat::RawBytesUnchecked;

/// Initial setup artifacts including trusted setup, proving key and verifying key.
#[derive(Clone, Debug)]
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

impl Setup {
    /// Serialize setup to raw bytes.
    pub fn to_bytes(self) -> Vec<u8> {
        let mut buffer = vec![];
        self.params
            .write_custom(&mut buffer, SERDE_FORMAT)
            .expect("Failed to save SRS");
        buffer.extend(self.pk.to_bytes(SERDE_FORMAT));
        buffer
    }

    /// Deserialize setup from raw bytes.
    pub fn from_bytes(buffer: &mut &[u8]) -> Self {
        let params =
            ParamsKZG::<Bn256>::read_custom(buffer, SERDE_FORMAT).expect("Failed to read SRS");
        let pk = ProvingKey::<G1Affine>::from_bytes::<StandardPlonk>(buffer, SERDE_FORMAT)
            .expect("Failed to read proving key");
        Self {
            k: params.k(),
            vk: pk.get_vk().clone(),
            pk,
            params,
        }
    }
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
