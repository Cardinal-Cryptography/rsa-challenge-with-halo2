use std::fs::read;

use anyhow::{Context, Result};
use contract_build::{
    BuildMode, ExecuteArgs, ManifestPath, OptimizationPasses, Verbosity, DEFAULT_MAX_MEMORY_PAGES,
};
use contract_extrinsics::{
    BalanceVariant, CallCommandBuilder, ExtrinsicOptsBuilder, InstantiateCommandBuilder,
};
use subxt::{
    config::{substrate::BlakeTwo256, Hasher},
    utils::AccountId32,
};

use crate::{get_contract_manifest, read_setup, PROOF_FILE};

pub fn run_contract_build() -> Result<()> {
    println!("⏳ Building contract...");
    contract_build::execute(ExecuteArgs {
        manifest_path: ManifestPath::new(get_contract_manifest().into())?,
        build_mode: BuildMode::Release,
        optimization_passes: Some(OptimizationPasses::default()),
        max_memory_pages: DEFAULT_MAX_MEMORY_PAGES,
        ..Default::default()
    })?;
    println!("✅ Contract built");
    Ok(())
}

pub async fn run_contract_deployment(challenge: u128, reward: u128) -> Result<()> {
    println!("⏳ Deploying contract...");

    let setup = read_setup()?;
    let vk_hash = BlakeTwo256::hash(&setup.serialize_vk());

    let command = InstantiateCommandBuilder::default()
        .args(vec![challenge.to_string(), format!("{vk_hash:?}")])
        .value(BalanceVariant::Default(reward))
        .extrinsic_opts(
            ExtrinsicOptsBuilder::default()
                .suri("//Alice")
                .manifest_path(Some(get_contract_manifest()))
                .done(),
        )
        .done()
        .await?;

    println!("⏳ Instantiating contract...");
    let result = command.instantiate(None).await.unwrap();
    println!(
        "✅ Contract deployed at address: \x1b[1m{}\x1b[0m",
        result.contract_address
    );
    Ok(())
}

pub async fn run_submission(address: AccountId32) -> Result<()> {
    println!("⏳ Submitting solution...");
    let proof = read(PROOF_FILE).context("Failed to read SNARK proof")?;
    println!("✅ Loaded SNARK proof from `{PROOF_FILE}`");

    let command = CallCommandBuilder::default()
        .contract(address)
        .message("solve")
        .args(vec![format!("{proof:?}")])
        .extrinsic_opts(
            ExtrinsicOptsBuilder::default()
                .suri("//Alice")
                .manifest_path(Some(get_contract_manifest()))
                .done(),
        )
        .done()
        .await?;

    println!("⏳ Calling contract...");
    let events = command.call(None).await.unwrap();
    println!("✅ Contract called");

    let event_log = events.display_events(Verbosity::Default, command.token_metadata())?;
    if event_log.contains("ChallengeSolved") {
        println!("✅ \x1b[1mChallenge solved!\x1b[0m");
    } else {
        println!("❌ \x1b[1mChallenge not solved, proof found to be incorrect\x1b[0m");
    }
    Ok(())
}
