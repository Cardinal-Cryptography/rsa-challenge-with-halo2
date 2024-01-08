use std::error::Error;

use drink::{
    runtime::RuntimeWithContracts,
    session::{Session, NO_ENDOWMENT, NO_SALT},
    AccountId32,
};
use frame_support::traits::fungible::Inspect;
use runtimes::RuntimeWithAcceptingCE;

use crate::tests::runtimes::RuntimeWithRejectingCE;

const CHALLENGE: u128 = 41 * 43;
const VK_ID: [u8; 32] = [0; 32];
const REWARD: u128 = 1_000_000;
const PROOF: &[&str] = &["[0, 1, 2, 3]"];

/// The account that will be used as a participant in the tests (the one that submits solutions).
const PARTICIPANT: AccountId32 = AccountId32::new([2; 32]);

#[drink::contract_bundle_provider]
enum BundleProvider {}

/// Deploy the contract and return a `drink::Session` object with `PARTICIPANT` set to be the caller.
///
/// Deployment is done by the Runtime's default account (potentially different than `PARTICIPANT`).
fn setup<Runtime: RuntimeWithContracts>() -> Result<Session<Runtime>, Box<dyn Error>>
where
    <<Runtime as drink::pallet_contracts::Config>::Currency as Inspect<
        <Runtime as frame_system::Config>::AccountId,
    >>::Balance: From<u128>,
    <Runtime as frame_system::Config>::AccountId: From<AccountId32>,
{
    let mut session = Session::<Runtime>::new()?;

    session.deploy_bundle(
        BundleProvider::local()?,
        "new",
        &[CHALLENGE.to_string(), format!("{VK_ID:?}")],
        NO_SALT,
        Some(REWARD.into()),
    )?;

    session.set_actor(PARTICIPANT.clone().into());
    Ok(session)
}

/// Simulate a positive scenario, i.e.:
/// - deploy the contract,
/// - submit a 'correct' proof and get the reward.
#[drink::test]
fn report_correct_solution_and_win() -> Result<(), Box<dyn Error>> {
    let mut session = setup::<RuntimeWithAcceptingCE>()?;

    let winner_balance_before = session.sandbox().free_balance(&PARTICIPANT);
    let _termination_result = session.call::<_, ()>("solve", PROOF, NO_ENDOWMENT);
    let winner_balance_after = session.sandbox().free_balance(&PARTICIPANT);

    // We check the lowerbound, as the exact reward will be enlarged by the storage deposit.
    assert!(winner_balance_before + REWARD <= winner_balance_after);
    Ok(())
}

/// Simulate a negative scenario, i.e.:
/// - deploy the contract,
/// - submit an 'incorrect' proof and assert, that the reward wasn't paid
#[drink::test]
fn report_incorrect_solution_and_win() -> Result<(), Box<dyn Error>> {
    let mut session = setup::<RuntimeWithRejectingCE>()?;

    let winner_balance_before = session.sandbox().free_balance(&PARTICIPANT);
    session.call::<_, ()>("solve", PROOF, NO_ENDOWMENT)??;
    let winner_balance_after = session.sandbox().free_balance(&PARTICIPANT);

    assert_eq!(winner_balance_before, winner_balance_after);
    Ok(())
}

mod runtimes {
    pub use accepting_runtime::RuntimeWithAcceptingCE;
    pub use rejecting_runtime::RuntimeWithRejectingCE;

    mod accepting_runtime {
        drink::create_minimal_runtime!(
            RuntimeWithAcceptingCE,
            crate::tests::extension_mocks::AlwaysAcceptExtension
        );
    }
    mod rejecting_runtime {
        drink::create_minimal_runtime!(
            RuntimeWithRejectingCE,
            crate::tests::extension_mocks::AlwaysRejectExtension
        );
    }
}

mod extension_mocks {
    use baby_liminal_extension::{
        extension_ids::{EXTENSION_ID, VERIFY_FUNC_ID},
        status_codes::{VERIFY_INCORRECT_PROOF, VERIFY_SUCCESS},
    };
    use drink::pallet_contracts::chain_extension::{
        ChainExtension, Config as ContractsConfig, Environment, Ext, InitState, Result, RetVal,
    };

    /// A chain extension that will always claim that the SNARK proof is correct.
    #[derive(Default)]
    pub struct AlwaysAcceptExtension;

    impl<Runtime: ContractsConfig> ChainExtension<Runtime> for AlwaysAcceptExtension {
        fn call<E: Ext<T = Runtime>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal> {
            assert!(env.ext_id() == EXTENSION_ID && env.func_id() == VERIFY_FUNC_ID);
            Ok(RetVal::Converging(VERIFY_SUCCESS))
        }
    }

    /// A chain extension that will always claim that the SNARK proof is incorrect.
    #[derive(Default)]
    pub struct AlwaysRejectExtension;

    impl<Runtime: ContractsConfig> ChainExtension<Runtime> for AlwaysRejectExtension {
        fn call<E: Ext<T = Runtime>>(&mut self, env: Environment<E, InitState>) -> Result<RetVal> {
            assert!(env.ext_id() == EXTENSION_ID && env.func_id() == VERIFY_FUNC_ID);
            Ok(RetVal::Converging(VERIFY_INCORRECT_PROOF))
        }
    }
}
