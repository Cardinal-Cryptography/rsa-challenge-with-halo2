use anyhow::Result;
use subxt::{dynamic::Value, ext::scale_value::Composite, OnlineClient, PolkadotConfig};

use crate::read_setup;

const PALLET_NAME: &str = "VkStorage";
const EXTRINSIC_NAME: &str = "store_key";

pub async fn run_vk_registration() -> Result<()> {
    println!("⏳ Preparing for verification key registration...");
    let vk_bytes = read_setup()?.serialize_vk();

    let api = OnlineClient::<PolkadotConfig>::new().await?;
    let call = subxt::dynamic::tx(
        PALLET_NAME,
        EXTRINSIC_NAME,
        Composite::unnamed([Value::from_bytes(&vk_bytes)]),
    );

    println!("⏳ Registering verification key...");
    api.tx()
        .sign_and_submit_then_watch_default(&call, &subxt_signer::sr25519::dev::alice())
        .await?
        .wait_for_finalized_success()
        .await?;
    println!("✅ Registered verification key");
    Ok(())
}
