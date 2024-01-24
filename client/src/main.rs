use std::{
    fs::read,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use clap::Parser;
use rsa_circuit::utils::Setup;
use subxt::utils::AccountId32;

use crate::{
    chain_ops::run_vk_registration,
    circuit_ops::{run_proof_generation, run_snark_setup},
    command::Command,
    contract_ops::{run_contract_build, run_contract_deployment, run_submission},
};

const CIRCUIT_MAX_K: u32 = 5;
const SNARK_SETUP_FILE: &str = "snark-setup";
const PROOF_FILE: &str = "submission-data";

mod chain_ops;
mod circuit_ops;
mod command;
mod contract_ops;
mod signer;

fn read_setup() -> Result<Setup> {
    let setup_serialized = read(SNARK_SETUP_FILE).context("Failed to read SNARK setup")?;
    let setup = Setup::from_bytes(&mut setup_serialized.as_slice());
    println!("✅ Loaded SNARK setup from `{SNARK_SETUP_FILE}`");
    Ok(setup)
}

#[tokio::main]
async fn main() -> Result<()> {
    match Command::parse() {
        Command::SetupSnark => run_snark_setup()?,
        Command::GenerateProof { p, q, phrase } => run_proof_generation(p, q, phrase)?,
        Command::RegisterVk { url, phrase } => run_vk_registration(url, phrase).await?,
        Command::BuildContract => run_contract_build()?,
        Command::DeployContract {
            challenge,
            reward,
            url,
            phrase,
        } => run_contract_deployment(challenge, reward, url, phrase).await?,
        Command::SubmitSolution {
            address,
            url,
            phrase,
        } => run_submission(AccountId32::from_str(&address)?, url, phrase).await?,
    }
    Ok(())
}

fn get_contract_manifest() -> impl Into<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../rsa_contract/Cargo.toml")
}
