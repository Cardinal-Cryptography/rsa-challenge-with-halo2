use clap::Parser;
use rsa_circuit::utils::generate_setup;

use crate::command::Command;

const CIRCUIT_MAX_K: u32 = 5;

mod command;

fn main() {
    match Command::parse() {
        Command::SetupSnark => {
            let setup = generate_setup(CIRCUIT_MAX_K);
        }
    }
}
