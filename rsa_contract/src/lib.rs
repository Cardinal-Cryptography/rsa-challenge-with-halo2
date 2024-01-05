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

    #[ink(storage)]
    pub struct RsaContract {
        /// The number to factorize.
        n: u128,
        /// Verification key identifier.
        vk_id: Hash,
    }

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
        pub fn solve(&mut self) {
            // We have to perform a trivial conversion between hash types (`KeyHash` cannot be stored directly in a
            // contract storage).
            let vk_id = KeyHash::from_slice(self.vk_id.as_ref());

            // If the verification succeeds, pay the reward and terminate contract.
            if let Ok(()) =
                self.env()
                    .extension()
                    .verify(vk_id, Default::default(), Default::default())
            {
                let winner = self.env().caller();
                self.env().terminate_contract(winner);
            }
        }
    }
}
