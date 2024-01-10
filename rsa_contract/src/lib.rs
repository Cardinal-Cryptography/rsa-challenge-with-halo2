#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[cfg(test)]
mod tests;

/// A contract representing RSA challenge. It has a single method `solve` that can be called by anyone. If the solution
/// is correct, the caller will be rewarded and the contract will terminate.
///
/// Proof verification is outsourced to a chain extension.
#[ink::contract(env = baby_liminal_extension::Environment)]
pub mod rsa_contract {
    use baby_liminal_extension::KeyHash;
    use ink::prelude::vec::Vec;

    #[ink(storage)]
    pub struct RsaContract {
        /// The number to factorize.
        n: u128,
        /// Verification key identifier.
        vk_id: Hash,
    }

    #[ink(event)]
    pub struct ChallengeSolved;
    #[ink(event)]
    pub struct ChallengeStillTooHard;

    impl RsaContract {
        /// Creates a new RSA challenge contract.
        ///
        /// # Arguments
        ///
        /// * `n` - The number to factorize.
        /// * `vk_id` - Verification key identifier.
        ///
        /// The transferred tokens, together with the storage deposit will become the reward for the first successful
        /// factorization.
        #[ink(constructor, payable)]
        pub fn new(n: u128, vk_id: Hash) -> Self {
            Self { n, vk_id }
        }

        /// Report solution.
        #[ink(message)]
        pub fn solve(&mut self, proof: Vec<u8>) {
            // We have to perform a trivial conversion between hash types (`KeyHash` cannot be stored directly in a
            // contract storage).
            let vk_id = KeyHash::from_slice(self.vk_id.as_ref());

            // If the verification succeeds, pay the reward and terminate contract.
            if let Ok(()) = self
                .env()
                .extension()
                .verify(vk_id, proof, self.prepare_public_input())
            {
                self.env().emit_event(ChallengeSolved);

                let winner = self.env().caller();
                self.env().terminate_contract(winner);
            } else {
                self.env().emit_event(ChallengeStillTooHard);
            }
        }

        /// Prepares the public input for the SNARK proof, which includes the number to factorize and the caller's
        /// address (to prevent front-running attacks).
        fn prepare_public_input(&self) -> Vec<u8> {
            let mut input = Vec::new();

            // First input is the number to factorize.
            input.extend(self.n.to_le_bytes());
            input.extend([0u8; 16]); // `Fr` elements are 256-bit, so we need to pad the input.

            // The second one is the caller's address. Since it might not be always convertible to `Fr`, we split it
            // into two 128-bit chunks.
            let caller = self.env().caller();
            let caller_bytes: &[u8; 32] = caller.as_ref();
            input.extend(u128::from_le_bytes(caller_bytes[..16].try_into().unwrap()).to_le_bytes());
            input.extend([0u8; 16]); // `Fr` elements are 256-bit, so we need to pad the input.
            input.extend(u128::from_le_bytes(caller_bytes[16..].try_into().unwrap()).to_le_bytes());
            input.extend([0u8; 16]); // `Fr` elements are 256-bit, so we need to pad the input.

            input
        }
    }
}
