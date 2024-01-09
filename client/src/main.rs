use std::fs::{read, write};

use clap::Parser;
use rsa_circuit::utils::{generate_proof, generate_setup, Account, Setup};

use crate::command::Command;

const CIRCUIT_MAX_K: u32 = 5;
const SNARK_SETUP_FILE: &str = "snark-setup";
const PROOF_FILE: &str = "submission-data";
const ACCOUNT: Account = [0; 32];

mod command;

fn main() {
    match Command::parse() {
        Command::SetupSnark => {
            let setup = generate_setup(CIRCUIT_MAX_K);
            write(SNARK_SETUP_FILE, setup.to_bytes()).expect("Failed to save SNARK setup");
        }
        Command::GenerateProof { p, q } => {
            let setup_serialized = read(SNARK_SETUP_FILE).expect("Failed to read SNARK setup");
            let setup = Setup::from_bytes(&mut setup_serialized.as_slice());

            let proof = generate_proof(&setup, p, q, ACCOUNT);
            write(PROOF_FILE, proof.to_bytes()).expect("Failed to save SNARK proof");
        }
    }
}
