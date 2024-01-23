# RSA challenge with halo2 and Aleph Zero

_A brief tutorial on how to write, deploy and interact with smart contracts utilizing ZK proofs on Aleph Zero blockchain._

Our working example is _RSA challenge_ game.
The idea is that we have a number `n` which is a product of two primes `p` and `q`.
`n` is publicly known, but `p` and `q` are secret.
The challenge is to factorize `n`.
The person, who solves this riddle first, wins the game and gets some reward.

# Contract

In the [rsa_contract](./rsa_contract) directory you can find the contract crate.
It represents the challenge as an ink! smart contract.
Once deployed, it allows anyone to submit a solution to the challenge.
The first accepted solution wins the reward and terminates the contract.

Since a blockchain is a public ledger, we have to hide the solution from the public, so that nobody can steal our credit.
To do so, we use a ZK proofs. A participant doesn't send `p` and `q` directly, but instead sends a proof that they know such `p` and `q` that factorize `n`.

The only thing that the contract has to do, is to verify such a proof.
Since this can be a very expensive operation in WASM, we outsource this heavy computation to the on-chain verifier, that should be much more efficient.
For curious minds, the verification is outsourced via a _chain extension_.

### Front-running prevention

Even when using ZK proofs, there is still a possibility of front-running.
Imagine that we are sending a proof that we know `p` and `q` that factorize `n`.
Now, anybody can just copy this proof and send it to the contract before us, stealing our reward!.

To prevent this, we additionally include our own public key in the proof.
This way, the contract can also verify that indeed it was the caller, who should get the reward.

# Circuit

The [rsa_circuit](./rsa_circuit) directory contains the circuit crate.
It is written with Aleph Zero's halo2 fork.
It also exposes some utilities for generating proofs and data serialization (please notice, that some conventions that are expected from the on-chain verifier are sometimes still very implicit).

# Client (Local Node)

In the [client](./client) directory you can find a simple CLI that interacts with the whole system.
Here we assume that a local node is running and exposes a ws endpoint `ws://localhost:9944` (see the [Instructions](#launching-a-local-chain) below). If you want to use a public zk devnet instead, try the instruction in the next section).
We can run an experiment as follows:

```bash
cd client/
# We build the client.
cargo build --release

# We generate the SNARK setup (SRS, proving and verifying keys).
./target/release/client setup-snark

⏳ Generating SNARK setup...
✅ Generated SNARK setup
💾 Saved SNARK setup to `snark-setup`

# We register the verification key in the vk-storage pallet.
./target/release/client register-vk

⏳ Preparing for verification key registration...
✅ Loaded SNARK setup from `snark-setup`
⏳ Registering verification key...
✅ Registered verification key

# We build the contract.
./target/release/client build-contract

⏳ Building contract...
 [==] Checking clippy linting rules
    Finished dev [unoptimized + debuginfo] target(s) in 0.32s
 [==] Building cargo project
    Finished release [optimized] target(s) in 0.24s
✅ Contract built

# We deploy the RSA contract. First argument is the number to factorize, second is the reward.
./target/release/client deploy-contract 1763 1000000000

⏳ Deploying contract...
✅ Loaded SNARK setup from `snark-setup`
⏳ Instantiating contract...
✅ Contract deployed at address: 5G4Z7MY2jf1rfF63mECiSTUPVSp7S9kH69fgFBF3Aj8uBxwM

# We generate a proof for the factors 41 and 43.
./target/release/client generate-proof 41 43

⏳ Preparing for SNARK proof generation...
✅ Loaded SNARK setup from `snark-setup`
⏳ Generating SNARK proof...
✅ Generated SNARK proof
💾 Saved SNARK proof to `submission-data`

# We submit the proof to the contract. We have to pass the contract address as an argument (it was printed after the deployment).
./target/release/client submit-solution 5G4Z7MY2jf1rfF63mECiSTUPVSp7S9kH69fgFBF3Aj8uBxwM

⏳ Submitting solution...
✅ Loaded SNARK proof from `submission-data`
⏳ Calling contract...
✅ Contract called
✅ Challenge solved!
```

> Note: all the chain and contract interactions are done with seed phrase `//Alice`.

In case our proof is invalid, we will get an error:

```bash
⏳ Submitting solution...
✅ Loaded SNARK proof from `submission-data`
⏳ Calling contract...
✅ Contract called
❌ Challenge not solved, proof found to be incorrect
```

# Client (ZK Devnet)

We repeat the above instructions, but this time running on a public devnet (no local node required). The developer dashboard for this devnet is available under the link: https://dev.azero.dev/?rpc=wss%3A%2F%2Fws-fe-zk.dev.azero.dev#/explorer note that this is running with a custom ws endpoint `wss://ws-fe-zk.dev.azero.dev/`.

 In order to succeed, you must generate an account and save the seed phrase (12 words) (we will assume you keep it in `YOUR_PHRASE` variable). Then you must make sure that your new account has funds, by getting them from the faucet available at https://faucet-fe-zk.dev.azero.dev/. If you have trouble with any of the steps, please visit the builders channel in Aleph Zero discord https://discord.com/invite/alephzero.

 We provide a short summary of the steps given in the previous section.

```bash
cd client/
# We build the client.
cargo build --release

# We generate the SNARK setup (SRS, proving and verifying keys).
./target/release/client setup-snark

# We register the verification key in the vk-storage pallet.
./target/release/client register-vk --phrase=YOUR_PHRASE --url=wss://ws-fe-zk.dev.azero.dev/


# We build the contract.
./target/release/client build-contract

# We deploy the RSA contract. First argument is the number to factorize, second is the reward.
./target/release/client deploy-contract 1763 1000000000 --phrase=YOUR_PHRASE --url=wss://ws-fe-zk.dev.azero.dev/

# We generate a proof for the factors 41 and 43.
./target/release/client generate-proof 41 43 --phrase=YOUR_PHRASE

# We submit the proof to the contract. We have to pass the contract address as an argument (it was printed after the deployment).
./target/release/client submit-solution CONTRACT_ADDRESS

```

In case our proof is invalid, we will get an error:

```bash
⏳ Submitting solution...
✅ Loaded SNARK proof from `submission-data`
⏳ Calling contract...
✅ Contract called
❌ Challenge not solved, proof found to be incorrect
```



# Exploiting the contract

The circuit itself has a little, but critical bug 🐛.
It is possible to generate a valid proof for a wrong solution, that will be accepted by the contract.
We encourage you to find this exploit and steal the reward!

# Launching a local chain

In order to launch a local Aleph Zero chain with ZK utilities, you can either:
- run a single-node chain with Docker: simply run `./launch-chain.sh`,
- or run `./scripts/run_nodes.sh --liminal` on the `main` branch from [`aleph-zero`](https://github.com/Cardinal-Cryptography/aleph-node) repository.

Mac users might experience some networking issues with the Docker image, so we recommend the latter option.
