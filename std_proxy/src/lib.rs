use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, near_bindgen, AccountId, PromiseOrValue};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Prepaid gas for making a single simple call.
const SINGLE_CALL_GAS: u64 = 200_000_000_000_000;

#[ext_contract(ext)]
pub trait StdRef {
    fn get_reference_data(&self, base: String, quote: String) -> Option<(u128, u64, u64)>;
    fn get_reference_data_bulk(
        &self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Option<Vec<(u128, u64, u64)>>;
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StdProxy {
    pub ref_: AccountId,
    pub owner: AccountId,
}

#[near_bindgen]
impl StdProxy {
    #[init]
    pub fn new(ref_: AccountId) -> Self {
        assert!(!env::state_exists(), "ALREADY_INITIALIZED");
        Self {
            ref_: ref_,
            owner: env::signer_account_id(),
        }
    }

    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn transfer_ownership(&mut self, new_owner: AccountId) {
        assert!(env::signer_account_id() == self.get_owner(), "NOT_AN_OWNER");
        env::log(format!("transfer ownership from {} to {}", self.owner, new_owner).as_bytes());
        self.owner = new_owner;
    }

    pub fn get_ref(&self) -> AccountId {
        self.ref_.clone()
    }

    pub fn set_ref(&mut self, new_ref: AccountId) {
        assert!(env::signer_account_id() == self.get_owner(), "NOT_AN_OWNER");
        env::log(format!("set ref from {} to {}", self.ref_, new_ref).as_bytes());
        self.ref_ = new_ref
    }

    pub fn get_reference_data(
        &mut self,
        base: String,
        quote: String,
    ) -> PromiseOrValue<Option<(u128, u64, u64)>> {
        ext::get_reference_data(base, quote, &self.ref_, 0, SINGLE_CALL_GAS).into()
    }

    pub fn get_reference_data_bulk(
        &mut self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> PromiseOrValue<Option<Vec<(u128, u64, u64)>>> {
        ext::get_reference_data_bulk(bases, quotes, &self.ref_, 0, SINGLE_CALL_GAS).into()
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // // part of writing unit tests is setting up a mock context
    // // in this example, this is only needed for env::log in the contract
    // // this is also a useful list to peek at when wondering what's available in env::*
    // fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
    //     VMContext {
    //         current_account_id: "alice.testnet".to_string(),
    //         signer_account_id: "robert.testnet".to_string(),
    //         signer_account_pk: vec![0, 1, 2],
    //         predecessor_account_id: "jane.testnet".to_string(),
    //         input,
    //         block_index: 0,
    //         block_timestamp: 0,
    //         account_balance: 0,
    //         account_locked_balance: 0,
    //         storage_usage: 0,
    //         attached_deposit: 0,
    //         prepaid_gas: 10u64.pow(18),
    //         random_seed: vec![0, 1, 2],
    //         is_view,
    //         output_data_receivers: vec![],
    //         epoch_height: 19,
    //     }
    // }

    // // mark individual unit tests with #[test] for them to be registered and fired
    // #[test]
    // fn increment() {
    //     // set up the mock context into the testing environment
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     // instantiate a contract variable with the counter at zero
    //     let mut contract = Counter { val: 0 };
    //     contract.increment();
    //     println!("Value after increment: {}", contract.get_num());
    //     // confirm that we received 1 when calling get_num
    //     assert_eq!(1, contract.get_num());
    // }

    // #[test]
    // fn decrement() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut contract = Counter { val: 0 };
    //     contract.decrement();
    //     println!("Value after decrement: {}", contract.get_num());
    //     // confirm that we received -1 when calling get_num
    //     assert_eq!(-1, contract.get_num());
    // }

    // #[test]
    // fn increment_and_reset() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut contract = Counter { val: 0 };
    //     contract.increment();
    //     contract.reset();
    //     println!("Value after reset: {}", contract.get_num());
    //     // confirm that we received -1 when calling get_num
    //     assert_eq!(0, contract.get_num());
    // }
}