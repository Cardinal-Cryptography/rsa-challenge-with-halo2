#[derive(clap::Parser)]
pub enum Command {
    // ------------ LOCAL CIRCUIT-RELATED OPERATIONS -------------------------------------------------------------------
    /// Run trusted setup phase and circuit-specific processing. Write the result (SRS, proving key, verifying key) to
    /// a file.
    SetupSnark,
    /// Generate a proof for the given advices and write it to a file.
    GenerateProof {
        p: u128,
        q: u128,
        #[clap(long, default_value = "//Alice")]
        phrase: String,
    },

    // ------------ CHAIN OPERATIONS -----------------------------------------------------------------------------------
    /// Register verifying key on the blockchain.
    RegisterVk {
        #[clap(long, default_value = "ws://localhost:9944")]
        url: url::Url,
        #[clap(long, default_value = "//Alice")]
        phrase: String,
    },

    // ------------ CONTRACT OPERATIONS --------------------------------------------------------------------------------
    BuildContract,
    DeployContract {
        challenge: u128,
        reward: u128,
        #[clap(long, default_value = "ws://localhost:9944")]
        url: url::Url,
        #[clap(long, default_value = "//Alice")]
        phrase: String,
    },
    SubmitSolution {
        address: String,
        #[clap(long, default_value = "ws://localhost:9944")]
        url: url::Url,
        #[clap(long, default_value = "//Alice")]
        phrase: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Command::command().debug_assert()
    }
}
