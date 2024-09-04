// token.rs

use candid::{CandidType, Deserialize};
use std::cell::RefCell;
use crate::TransferError;

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct Token {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: RefCell<u128>,
}

impl Token {
    pub fn new(name: String, symbol: String, decimals: u8, initial_supply: u128) -> Self {
        Self {
            name,
            symbol,
            decimals,
            total_supply: RefCell::new(initial_supply),
        }
    }

    pub fn mint(&self, amount: u128) -> Result<(), TransferError> {
        let mut total_supply = self.total_supply.borrow_mut();
        *total_supply = total_supply.checked_add(amount).ok_or(TransferError::OverflowError)?;
        Ok(())
    }

    pub fn burn(&self, amount: u128) -> Result<(), TransferError> {
        let mut total_supply = self.total_supply.borrow_mut();
        *total_supply = total_supply.checked_sub(amount).ok_or(TransferError::OverflowError)?;
        Ok(())
    }

    // pub fn get_total_supply(&self) -> u128 {
    //     *self.total_supply.borrow()
    // }
}
