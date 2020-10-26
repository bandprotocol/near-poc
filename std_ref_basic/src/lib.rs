use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, near_bindgen, AccountId};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

static E9: u128 = 1_000_000_000;

macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.iter().map(|v| v.clone()).zip(zip!($($y.clone()), +))
    )
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct StdReferenceBasic {
    pub refs: UnorderedMap<String, (u128, u64, u64)>,
    pub owner: AccountId,
}

#[near_bindgen]
impl StdReferenceBasic {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "ALREADY_INITIALIZED");
        Self {
            refs: UnorderedMap::new(b"refs".to_vec()),
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

    pub fn get_refs(&self, symbol: String) -> Option<(u128, u64, u64)> {
        match &symbol[..] {
            "USD" => Some((E9, env::block_timestamp(), 0)),
            _ => self.refs.get(&symbol),
        }
    }

    pub fn get_reference_data(&self, base: String, quote: String) -> Option<(u128, u64, u64)> {
        match (self.get_refs(base.clone()), self.get_refs(quote.clone())) {
            (Some((br, bt, _)), Some((qr, qt, _))) => return Some((br * E9 * E9 / qr, bt, qt)),
            (None, Some(_)) => env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {}", base).as_bytes()),
            (Some(_), None) => {
                env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {}", quote).as_bytes())
            }
            _ => env::log(format!("REF_DATA_NOT_AVAILABLE_FOR: {} and {}", base, quote).as_bytes()),
        }
        None
    }

    pub fn get_reference_data_bulk(
        &self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Option<Vec<(u128, u64, u64)>> {
        assert!(bases.len() == quotes.len(), "BAD_INPUT_LENGTH");
        bases
            .iter()
            .zip(quotes.iter())
            .map(|(b, q)| self.get_reference_data(b.clone(), q.clone()))
            .collect()
    }

    pub fn relay(
        &mut self,
        symbols: Vec<String>,
        rates: Vec<u128>,
        resolve_times: Vec<u64>,
        request_ids: Vec<u64>,
    ) {
        assert!(env::signer_account_id() == self.get_owner(), "NOT_AN_OWNER");

        let len = symbols.len();
        assert!(rates.len() == len, "BAD_RATES_LENGTH");
        assert!(resolve_times.len() == len, "BAD_RESOLVE_TIMES_LENGTH");
        assert!(request_ids.len() == len, "BAD_REQUEST_IDS_LENGTH");

        for (s, (r, (rt, rid))) in zip!(&symbols, &rates, &resolve_times, &request_ids) {
            self.refs.insert(&s, &(r, rt, rid));
            env::log(format!("relay: {},{},{},{}", s, r, rt, rid).as_bytes());
        }
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
