//! Helpers for working with the RSA circuit.

use halo2_proofs::{
    arithmetic::Field,
    halo2curves::{
        bn256::{Bn256, Fr, G1Affine},
        ff::PrimeField,
    },
    plonk::{create_proof, keygen_pk, keygen_vk, ProvingKey, VerifyingKey},
    poly::{
        commitment::Params,
        kzg::{commitment::ParamsKZG, multiopen::ProverGWC},
    },
    standard_plonk::StandardPlonk,
    transcript::{Blake2bWrite, Challenge255, TranscriptWriterBuffer},
    SerdeFormat,
};
use rand::rngs::OsRng;

use crate::RsaChallenge;

const SERDE_FORMAT: SerdeFormat = SerdeFormat::RawBytesUnchecked;

/// Type representing an identifier of the participant.
pub type Account = [u8; 32];

/// Initial setup artifacts including trusted setup result, proving key and verifying key.
#[derive(Clone, Debug)]
pub struct Setup {
    /// Logarithm of the maximum number of rows in the PLONK table (polynomial degree).
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

    /// Serialize verifying key to raw bytes as it is expected by the on-chain verifier.
    pub fn serialize_vk(&self) -> Vec<u8> {
        let mut buffer = vec![];
        buffer.extend(self.k.to_le_bytes());
        buffer.extend(self.vk.to_bytes(SERDE_FORMAT));
        buffer
    }
}

/// Run the initial setup phase (for SRS) and circuit processing (for keys).
pub fn generate_setup(k: u32) -> Setup {
    let circuit = RsaChallenge::default();
    let params = ParamsKZG::<Bn256>::setup(k, ParamsKZG::<Bn256>::mock_rng());
    let vk = keygen_vk(&params, &circuit).expect("vk generation should not fail");
    let pk = keygen_pk(&params, vk.clone(), &circuit).expect("pk generation should not fail");
    Setup { k, pk, vk, params }
}

/// Convert the public input from human-readable form to a scalar array.
pub fn prepare_public_input(n: u128, account: Account) -> [Fr; 3] {
    [
        Fr::from_u128(n),
        Fr::from_u128(u128::from_le_bytes(account[..16].try_into().unwrap())),
        Fr::from_u128(u128::from_le_bytes(account[16..].try_into().unwrap())),
    ]
}

/// Generate proof given `setup`, `p`, `q` and `account`.
pub fn generate_proof(setup: &Setup, p: u128, q: u128, account: Account) -> Vec<u8> {
    let n = p * q;
    let circuit = RsaChallenge {
        p: Some(Fr::from_u128(p)),
        #[cfg(not(test))]
        p_dec_inv: Some(Fr::from_u128(p - 1).invert().unwrap()),
        #[cfg(test)]
        p_dec_inv: Some(Fr::from_u128(p - 1).invert().unwrap_or(Fr::zero())),
        q: Some(Fr::from_u128(q)),
        #[cfg(not(test))]
        q_dec_inv: Some(Fr::from_u128(q - 1).invert().unwrap()),
        #[cfg(test)]
        q_dec_inv: Some(Fr::from_u128(q - 1).invert().unwrap_or(Fr::zero())),
    };
    let instances = prepare_public_input(n, account);

    let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);
    create_proof::<_, ProverGWC<'_, Bn256>, _, _, _, _>(
        &setup.params,
        &setup.pk,
        &[circuit],
        &[&[&instances]],
        OsRng,
        &mut transcript,
    )
    .expect("Failed to generate proof");
    transcript.finalize()
}
