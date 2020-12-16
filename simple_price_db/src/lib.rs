use borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, near_bindgen, AccountId, PromiseOrValue};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! zip {
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => (
        $x.iter().map(|v| v.clone()).zip(zip!($($y.clone()), +))
    )
}

#[ext_contract(ext)]
pub trait ExtContract {
    fn get_reference_data(&self, base: String, quote: String) -> Option<(u128, u64, u64)>;
    fn get_reference_data_bulk(
        &self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) -> Option<Vec<(u128, u64, u64)>>;
    fn set_price(
        &self,
        symbol: String,
        #[callback]
        value_opt: Option<(u128, u64, u64)>,
    );
    fn set_price_multi(
        &self,
        symbols: Vec<String>,
        #[callback]
        values_opt: Vec<Option<(u128, u64, u64)>>,
    );
}

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct SimplePriceDB {
    pub oracle: AccountId,
    pub prices: UnorderedMap<String, u128>,
}

#[near_bindgen]
impl SimplePriceDB {
    #[init]
    pub fn new(oracle: AccountId) -> Self {
        assert!(!env::state_exists(), "ALREADY_INITIALIZED");
        Self { oracle, prices: UnorderedMap::new(b"prices".to_vec()) }
    }

    pub fn get_oracle(&self) -> AccountId {
        self.oracle.clone()
    }

    pub fn set_oracle(&mut self, new_oracle: AccountId) {
        env::log(format!("set oracle address from {} to {}", self.oracle, new_oracle).as_bytes());
        self.oracle = new_oracle
    }

    pub fn get_price(&self, symbol: String) -> Option<u128> {
        self.prices.get(&symbol)
    }

    pub fn save_price(&self, base: String, quote: String) {
        let prepaid_gas = env::prepaid_gas();
        let this = env::current_account_id();
        ext::get_reference_data(
            base.clone(),
            quote.clone(),
            &self.oracle,
            0,
            prepaid_gas / 4
        ).then(
            ext::set_price(format!("{}/{}",base, quote), &this, 0, prepaid_gas / 4)
        );
    }

    pub fn save_price_multi(
        &mut self,
        bases: Vec<String>,
        quotes: Vec<String>,
    ) {
        assert!(
            bases.len() == quotes.len(),
            format!("BASES_QUOTES_SIZE_IS_NOT_EQUAL:{}!={}",bases.len(),quotes.len())
        );

        let prepaid_gas = env::prepaid_gas();
        let this = env::current_account_id();
        let mut symbols = vec![String::from(""); bases.len()];
        for (i, (base, quote)) in bases.iter().zip(quotes.iter()).enumerate() {
            symbols[i] = format!("{}/{}", base, quote);
        }
        ext::get_reference_data_bulk(
            bases.clone(),
            quotes.clone(),
            &self.oracle,
            0,
            prepaid_gas / 4
        ).then(
            ext::set_price_multi(symbols, &this, 0, prepaid_gas / 4)
        );
    }

    #[result_serializer(borsh)]
    pub fn set_price(
        &mut self,
        symbol: String,
        #[callback]
        value_opt: Option<(u128, u64, u64)>,
    ) {
        match value_opt {
            Some((rate, _, _)) => {
                env::log(format!("Save rate {:?} to state", &rate).as_bytes());
                self.prices.insert(&symbol, &rate);
            },
            None => {
                env::log(format!("Got None from the oracle").as_bytes());
            }
        }
    }

    #[result_serializer(borsh)]
    pub fn set_price_multi(
        &mut self,
        symbols: Vec<String>,
        #[callback]
        values_opt: Option<Vec<(u128, u64, u64)>>,
    ) {
        match values_opt {
            Some(values) => {
                for (symbol, (rate, _, _)) in zip!(&symbols, &values) {
                    self.prices.insert(&symbol, &rate);
                }
                env::log(format!("Got values {:?}", values).as_bytes());
            },
            None => {
                env::log(format!("Got None").as_bytes());
            }
        }
    }

}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    fn alice() -> AccountId {
        "alice.near".to_string()
    }

    fn bob() -> AccountId {
        "bob.near".to_string()
    }

    fn carol() -> AccountId {
        "carol.near".to_string()
    }

    fn std_proxy() -> AccountId {
        "std_proxy.near".to_string()
    }

    fn get_context() -> VMContext {
        VMContext {
            current_account_id: alice(),
            signer_account_id: bob(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: carol(),
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    #[test]
    fn test_create_new_contract() {
        let context = get_context();
        testing_env!(context);
        let contract = SimplePriceDB::new(std_proxy());

        // check state
        assert_eq!(std_proxy(), contract.oracle);

        // check owner using view function
        assert_eq!(std_proxy(), contract.get_oracle());
    }
}
