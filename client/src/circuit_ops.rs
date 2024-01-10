use std::fs::write;

use anyhow::{Context, Result};
use rsa_circuit::utils::{generate_proof, generate_setup};
use subxt_signer::sr25519::dev::alice;

use crate::{read_setup, CIRCUIT_MAX_K, PROOF_FILE, SNARK_SETUP_FILE};

pub fn run_snark_setup() -> Result<()> {
    println!("⏳ Generating SNARK setup...");
    let setup = generate_setup(CIRCUIT_MAX_K);
    println!("✅ Generated SNARK setup");

    write(SNARK_SETUP_FILE, setup.to_bytes()).context("Failed to save SNARK setup")?;
    println!("💾 Saved SNARK setup to `{SNARK_SETUP_FILE}`");
    Ok(())
}

pub fn run_proof_generation(p: u128, q: u128) -> Result<()> {
    println!("⏳ Preparing for SNARK proof generation...");
    let setup = read_setup()?;

    let account = alice().public_key().to_account_id().0;
    println!("⏳ Generating SNARK proof...");
    let proof = generate_proof(&setup, p, q, account);
    println!("✅ Generated SNARK proof");

    write(PROOF_FILE, proof).context("Failed to save SNARK proof")?;
    println!("💾 Saved SNARK proof to `{PROOF_FILE}`");
    Ok(())
}
