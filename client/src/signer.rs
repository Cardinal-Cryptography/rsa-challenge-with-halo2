use std::str::FromStr;

use anyhow::Result;
use subxt_signer::{sr25519::Keypair, SecretUri};

pub(crate) fn signer_from_phrase(phrase: String) -> Result<Keypair> {
    let suri = SecretUri::from_str(&phrase)?;
    let signer = Keypair::from_uri(&suri)?;
    Ok(signer)
}
