// transfer.rs

use candid::Principal;
// use std::collections::HashMap;
// use std::cell::RefCell;
use crate::{Wallet, TransferError};
// use crate::get_time;

// Define the TransferManager trait
pub trait TransferManager {
    fn transfer(&self, from: Principal, to: Principal, amount: u128) -> Result<(), TransferError>;
}

// Implement a simple TransferManager
pub struct SimpleTransferManager;

impl SimpleTransferManager {
    pub fn new() -> Self {
        SimpleTransferManager
    }
}

impl TransferManager for SimpleTransferManager {
    fn transfer(&self, from: Principal, to: Principal, amount: u128) -> Result<(), TransferError> {
        // Access the global WALLETS
        crate::WALLETS.with(|wallets| {
            let mut wallets = wallets.borrow_mut();

            // Get sender's wallet
            let sender_wallet = wallets.get_mut(&from).ok_or(TransferError::SenderWalletNotFound)?;

            // Check if sender has enough balance
            let sender_balance = sender_wallet.get_balance("ICPT");
            if sender_balance < amount {
                return Err(TransferError::InsufficientBalance);
            }

            // Subtract the amount from sender
            sender_wallet.update_balance("ICPT", sender_balance - amount);

            // Get or create the recipient's wallet
            let recipient_wallet = wallets.entry(to).or_insert_with(|| Wallet::new(to));

            // Add the amount to recipient
            let recipient_balance = recipient_wallet.get_balance("ICPT");
            recipient_wallet.update_balance("ICPT", recipient_balance.checked_add(amount).ok_or(TransferError::OverflowError)?);

            Ok(())
        })
    }
}
