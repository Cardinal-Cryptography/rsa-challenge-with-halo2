use std::fs::{read, write};

use anyhow::{Context, Result};
use clap::Parser;
use rsa_circuit::utils::{generate_proof, generate_setup, Account, Setup};
use subxt::{dynamic::Value, ext::scale_value::Composite, OnlineClient, PolkadotConfig};

use crate::command::Command;

const CIRCUIT_MAX_K: u32 = 5;
const SNARK_SETUP_FILE: &str = "snark-setup";
const PROOF_FILE: &str = "submission-data";
const ACCOUNT: Account = [0; 32];

mod command;

fn read_setup() -> Result<Setup> {
    let setup_serialized = read(SNARK_SETUP_FILE).context("Failed to read SNARK setup")?;
    Ok(Setup::from_bytes(&mut setup_serialized.as_slice()))
}

#[tokio::main]
async fn main() -> Result<()> {
    match Command::parse() {
        Command::SetupSnark => {
            let setup = generate_setup(CIRCUIT_MAX_K);
            write(SNARK_SETUP_FILE, setup.to_bytes()).context("Failed to save SNARK setup")?;
        }
        Command::GenerateProof { p, q } => {
            let setup = read_setup()?;
            let proof = generate_proof(&setup, p, q, ACCOUNT);
            write(PROOF_FILE, proof.to_bytes()).context("Failed to save SNARK proof")?;
        }
        Command::RegisterVk => {
            let vk_bytes = read_setup()?.serialize_vk();

            let api = OnlineClient::<PolkadotConfig>::new().await?;
            let call = subxt::dynamic::tx(
                "VkStorage",
                "store_key",
                Composite::unnamed([Value::from_bytes(&vk_bytes)]),
            );

            api.tx()
                .sign_and_submit_then_watch_default(&call, &subxt_signer::sr25519::dev::alice())
                .await?
                .wait_for_finalized_success()
                .await?;
        }
    }
    Ok(())
}
