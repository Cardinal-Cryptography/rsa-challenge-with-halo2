//! Circuit for the RSA challenge.

#![deny(missing_docs)]

use halo2_proofs::{
    circuit::{Cell, Layouter, Region, Value},
    halo2curves::bn256::Fr,
    plonk::{Circuit, Column, ConstraintSystem, Error, Fixed},
    standard_plonk::{StandardPlonk, StandardPlonkConfig},
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
/// Since `account` is usually 256 bits long (`[u8; 32]`), we need to split it into two 128 bit chunks, so that we have
/// certainty that both can be safely decoded as `Fr` elements.
///
/// # Exploit
///
/// The relation has a bugt, which allows to satisfy the circuit with an invalid solution. Try to find it!
#[derive(Default)]
pub struct RsaChallenge {
    /// First prime factor of the challenge.
    p: Option<Fr>,
    /// Inverse of `p-1`.
    ///
    /// Required for the non-triviality check.
    p_dec_inv: Option<Fr>,
    /// Second prime factor of the challenge.
    q: Option<Fr>,
    /// Inverse of `q-1`.
    ///
    /// Required for the non-triviality check.
    q_dec_inv: Option<Fr>,
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
                let p_value = || Value::known(self.p.unwrap());
                let p_dec_inv_value = || Value::known(self.p_dec_inv.unwrap());
                let q_value = || Value::known(self.q.unwrap());
                let q_dec_inv_value = || Value::known(self.q_dec_inv.unwrap());

                // Check that `p*q = n`.
                let p_cell = region.assign_advice(|| "p", config.a, 0, p_value)?;
                let q_cell = region.assign_advice(|| "q", config.b, 0, q_value)?;
                Self::negate_at_selector(&mut region, config.q_ab, || "p*q", 0)?;

                // Zero out the rest of the instances just by negating them. This way we are ensuring that they will be
                // embedded into the proof (committed to).
                let ann1 = || "account low";
                region.assign_advice_from_instance(ann1, config.instance, 1, config.a, 1)?;
                Self::negate_at_selector(&mut region, config.q_a, ann1, 1)?;

                let ann2 = || "account_high";
                region.assign_advice_from_instance(ann2, config.instance, 2, config.a, 2)?;
                Self::negate_at_selector(&mut region, config.q_a, ann2, 2)?;

                // Check that both `p` and `q` are greater than 1.
                Self::check_non_triviality(
                    &mut region,
                    'p',
                    p_value,
                    p_dec_inv_value,
                    3,
                    p_cell.cell(),
                    &config,
                )?;
                Self::check_non_triviality(
                    &mut region,
                    'q',
                    q_value,
                    q_dec_inv_value,
                    4,
                    q_cell.cell(),
                    &config,
                )?;

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

    fn check_non_triviality(
        region: &mut Region<Fr>,
        // either 'p' or 'q'
        x_id: char,
        // either `p` or `q`
        x_val: impl Fn() -> Value<Fr>,
        // either `p_dec_inv` or `q_dec_inv`
        x_dec_inv_val: impl Fn() -> Value<Fr>,
        // which row to use for check
        offset: usize,
        // the first copy of `x` - should be equal-constrained
        original_cell: Cell,
        // column identifiers
        config: &StandardPlonkConfig<Fr>,
    ) -> Result<(), Error> {
        // x != 1 <=> (x-1) * (x-1)^-1 == 1 <=> x * (x-1)^-1 - (x-1)^-1 == 1

        // Copy advice `x` to a new cell and ensure equality constraint with the previous copy.
        let x_copy = region.assign_advice(|| x_id, config.a, offset, x_val)?;
        region.constrain_equal(original_cell, x_copy.cell())?;

        // Copy advice (x-1)^-1 to a new cell.
        region.assign_advice(|| format!("({x_id}-1)^-1"), config.b, offset, x_dec_inv_val)?;

        // In this gate we take (x-1)^-1 with `-1` multiplier...
        Self::negate_at_selector(region, config.q_b, || "(p-1)^-1", offset)?;
        // and we take `x` * `(x-1)^-1` with `1` multiplier.
        region.assign_fixed(
            || format!("{x_id} * ({x_id}-1)^-1"),
            config.q_ab,
            offset,
            || Value::known(Fr::one()),
        )?;

        // `x * (x-1)^-1 - (x-1)^-1` should be equal to `1`.
        region.assign_fixed(|| "1", config.constant, offset, || Value::known(-Fr::one()))?;

        Ok(())
    }
}
