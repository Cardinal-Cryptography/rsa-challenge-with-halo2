//! Circuit for the RSA challenge.

#![deny(missing_docs)]

use halo2_proofs::{
    circuit::{Layouter, Region, Value},
    halo2curves::bn256::Fr,
    plonk::{Circuit, Column, ConstraintSystem, Error, Fixed},
    standard_plonk::StandardPlonk,
};

#[cfg(test)]
mod tests;
pub mod utils;

/// Circuit representing the RSA challenge.
///
/// There are two advices (private inputs): `p`, and `q` and two instances (public inputs): `n` and `account`. The
/// values should satisfy: `p * q = n`. The `account` instance is artificially included in the circuit to prevent
/// front running attacks.
///
/// Since `account` is usually 256 bits long (`[u8 ;32]`), we need to split it into two 128 bit chunks, so that we have
/// certainty that both can be safely decoded as `Fr` elements.
#[derive(Default)]
pub struct RsaChallenge {
    /// First prime factor of the challenge.
    p: Option<Fr>,
    /// Second prime factor of the challenge.
    q: Option<Fr>,
}

impl Circuit<Fr> for RsaChallenge {
    type Config = <StandardPlonk as Circuit<Fr>>::Config;
    type FloorPlanner = <StandardPlonk as Circuit<Fr>>::FloorPlanner;

    fn without_witnesses(&self) -> Self {
        RsaChallenge::default()
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        StandardPlonk::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "",
            |mut region| {
                // Check that `p*q = n`.
                region.assign_advice(|| "p", config.a, 0, || Value::known(self.p.unwrap()))?;
                region.assign_advice(|| "q", config.b, 0, || Value::known(self.q.unwrap()))?;
                Self::negate_at_selector(&mut region, config.q_ab, || "p*q", 0)?;

                // Zero out the rest of the instances just by negating them. This way we are ensuring that they will be
                // embedded into the proof (committed to).
                let ann1 = || "account low";
                region.assign_advice_from_instance(ann1, config.instance, 1, config.a, 1)?;
                Self::negate_at_selector(&mut region, config.q_a, ann1, 1)?;

                let ann2 = || "account_high";
                region.assign_advice_from_instance(ann2, config.instance, 2, config.a, 2)?;
                Self::negate_at_selector(&mut region, config.q_a, ann2, 2)?;

                Ok(())
            },
        )
    }
}

impl RsaChallenge {
    fn negate_at_selector(
        region: &mut Region<Fr>,
        selector: Column<Fixed>,
        annotation: impl Fn() -> &'static str,
        offset: usize,
    ) -> Result<(), Error> {
        let negated = || Value::known(-Fr::one());
        let annotation = || format!("selector for {}", annotation());
        region.assign_fixed(annotation, selector, offset, negated)?;
        Ok(())
    }
}
