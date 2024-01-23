use anyhow::Result;
use subxt::{dynamic::Value, ext::scale_value::Composite, OnlineClient, PolkadotConfig};
use url::Url;

use crate::{read_setup, signer::signer_from_phrase};

const PALLET_NAME: &str = "VkStorage";
const EXTRINSIC_NAME: &str = "store_key";

pub async fn run_vk_registration(url: Url, phrase: String) -> Result<()> {
    println!("⏳ Preparing for verification key registration...");
    let vk_bytes = read_setup()?.serialize_vk();

    let api = OnlineClient::<PolkadotConfig>::from_url(url).await?;
    let call = subxt::dynamic::tx(
        PALLET_NAME,
        EXTRINSIC_NAME,
        Composite::unnamed([Value::from_bytes(&vk_bytes)]),
    );
    println!("⏳ Preparing Signer...");
    let signer = signer_from_phrase(phrase)?;

    println!("⏳ Registering verification key...");
    api.tx()
        .sign_and_submit_then_watch_default(&call, &signer)
        .await?
        .wait_for_finalized_success()
        .await?;
    println!("✅ Registered verification key");
    Ok(())
}
