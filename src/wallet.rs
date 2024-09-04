// wallet.rs

use candid::Principal;
use std::collections::HashMap;
// use crate::TransferError;

#[derive(Clone, Debug)]
pub struct Wallet {
    pub owner: Principal,
    pub balances: HashMap<String, u128>,
}


impl Wallet {
    pub fn new(owner: Principal) -> Self {
        Self {
            owner,
            balances: HashMap::new(),
        }
    }

    pub fn get_balance(&self, token_symbol: &str) -> u128 {
        *self.balances.get(token_symbol).unwrap_or(&0)
    }

    pub fn update_balance(&mut self, token_symbol: &str, amount: u128) {
        self.balances.insert(token_symbol.to_string(), amount);
    }
}
