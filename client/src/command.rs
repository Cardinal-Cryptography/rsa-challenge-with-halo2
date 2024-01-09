#[derive(clap::Parser)]
pub enum Command {
    // ------------ LOCAL CIRCUIT-RELATED OPERATIONS -------------------------------------------------------------------
    /// Run trusted setup phase and circuit-specific processing. Write the result (SRS, proving key, verifying key) to
    /// a file.
    SetupSnark,
    /// Generate a proof for the given advices and write it to a file.
    GenerateProof { p: u128, q: u128 },

    // ------------ CHAIN OPERATIONS -----------------------------------------------------------------------------------
    /// Register verifying key on the blockchain.
    RegisterVk,
}
