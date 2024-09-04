// lib.rs

mod token;
mod wallet;
mod transfer;

use ic_cdk::api::caller;
use ic_cdk_macros::*;
use wallet::Wallet;
use token::Token;
use transfer::TransferManager;
use candid::{CandidType, Deserialize};
use std::cell::RefCell;
use std::collections::HashMap;
use candid::Principal;

// Define the TransferEvent struct to log transfers
#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct TransferEvent {
    pub from: Principal,
    pub to: Principal,
    pub amount: u128,
    pub timestamp: u64,
}

// Define the TransferError enum for error handling
#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub enum TransferError {
    InsufficientBalance,
    RecipientWalletNotFound,
    SenderWalletNotFound,
    Unauthorized,
    InvalidAmount,
    OverflowError,
}

// Persistent storage using thread_local!
thread_local! {
    // Token information
    static TOKEN: RefCell<Token> = RefCell::new(Token::new(
        "ICP Token".to_string(),
        "ICPT".to_string(),
        8,
        1_000_000_000_000_000_000
    ));
    
    // Wallets mapping Principal to Wallet
    static WALLETS: RefCell<HashMap<Principal, Wallet>> = RefCell::new(HashMap::new());
    
    // Transfer events log
    static TRANSFER_EVENTS: RefCell<Vec<TransferEvent>> = RefCell::new(Vec::new());
    
    // Owner of the canister
    static OWNER: RefCell<Principal> = RefCell::new(Principal::anonymous());
}

// Initialization function to set the owner
#[init]
fn init() {
    OWNER.with(|owner| *owner.borrow_mut() = caller());
    ic_cdk::println!("Canister initialized with owner: {:?}", caller());
}

// Query to get the balance of a wallet
#[query]
fn get_balance(owner: Principal) -> u128 {
    WALLETS.with(|wallets| {
        let wallets = wallets.borrow();
        match wallets.get(&owner) {
            Some(wallet) => wallet.get_balance("ICPT"),
            None => 0,
        }
    })
}

// Query to get token information
#[query]
fn get_token_info() -> Token {
    TOKEN.with(|token| token.borrow().clone())
}

// Update to create a new wallet for the caller
#[update]
fn create_wallet() -> Result<Principal, String> {
    let caller = caller();
    println!("Creating wallet for caller: {:?}", caller);
    WALLETS.with(|wallets| {
        let mut wallets = wallets.borrow_mut();
        if wallets.contains_key(&caller) {
            println!("Wallet already exists for caller: {:?}", caller);
            Err("Wallet already exists".to_string())
        } else {
            let new_wallet = Wallet::new(caller);
            wallets.insert(caller, new_wallet);
            println!("Wallet created successfully for caller: {:?}", caller);
            Ok(caller)
        }
    })
}

// Update to transfer tokens from caller to another Principal
#[update]
fn transfer(to: Principal, amount: u128) -> Result<bool, TransferError> {
    let caller = caller();
    if amount == 0 {
        return Err(TransferError::InvalidAmount);
    }

    // Access the TransferManager from the transfer module
    let transfer_manager = transfer::SimpleTransferManager::new();

    // Perform the transfer
    transfer_manager.transfer(caller, to, amount)?;

    // Log the transfer event
    let event = TransferEvent {
        from: caller,
        to,
        amount,
        timestamp: get_time(),
    };
    TRANSFER_EVENTS.with(|events| {
        events.borrow_mut().push(event);
    });

    Ok(true)
}

// Update to mint new tokens (only owner can mint)
#[update]
fn mint(to: Principal, amount: u128) -> Result<bool, TransferError> {
    if !is_owner() {
        return Err(TransferError::Unauthorized);
    }

    if amount == 0 {
        return Err(TransferError::InvalidAmount);
    }

    // Mint tokens in the Token module
    TOKEN.with(|token| {
        let token = token.borrow_mut();
        match token.mint(amount) {
            Ok(_) => Ok(()),
            Err(_) => Err(TransferError::OverflowError),
        }
    })?;

    // Update the recipient's wallet balance
    WALLETS.with(|wallets| {
        let mut wallets = wallets.borrow_mut();
        let wallet = wallets.entry(to).or_insert_with(|| Wallet::new(to));
        let balance = wallet.get_balance("ICPT");
        wallet.update_balance("ICPT", balance.checked_add(amount).ok_or(TransferError::OverflowError)?);
        Ok(())
    })?;

    // Log the transfer event
    let event = TransferEvent {
        from: Principal::anonymous(), // Minting can be represented as coming from "anonymous"
        to,
        amount,
        timestamp: get_time(),
    };
    TRANSFER_EVENTS.with(|events| {
        events.borrow_mut().push(event);
    });

    Ok(true)
}


// Update to burn tokens from the caller's wallet
#[update]
fn burn(amount: u128) -> Result<bool, TransferError> {
    let caller = caller();

    ic_cdk::println!("Attempting to burn amount: {}", amount);

    if amount == 0 {
        return Err(TransferError::InvalidAmount);
    }

// Access and update the wallet
WALLETS.with(|wallets| {
  let mut wallets = wallets.borrow_mut();
  
  // Retrieve the wallet of the caller or return an error
  let wallet = wallets.get_mut(&caller).ok_or(TransferError::SenderWalletNotFound)?;
  
  // Get the current balance of the wallet
  let balance = wallet.get_balance("ICPT");

  // Check if the balance is sufficient to burn the requested amount
  if balance < amount {
      ic_cdk::println!(
          "Insufficient balance: available {}, trying to burn {}",
          balance,
          amount
      );
      return Err(TransferError::InsufficientBalance);
  }

  // Update the wallet balance by subtracting the amount
  wallet.update_balance("ICPT", balance.checked_sub(amount).ok_or(TransferError::OverflowError)?);

  // Return Ok(()) to indicate success
  Ok(())
})?;


// Update the total supply in the Token module
TOKEN.with(|token| {
  // Mutably borrow the token and perform the burn operation
  let token = token.borrow_mut();
  
  // Perform the burn operation and handle any errors that may occur
  token.burn(amount).map_err(|_| TransferError::OverflowError)?;
  
  // Return Ok(()) to indicate that the operation was successful
  Ok(())
})?;

    // Log the burn event
    let event = TransferEvent {
        from: caller,
        to: Principal::anonymous(), // Burning can be represented as sending to "anonymous"
        amount,
        timestamp: get_time(),
    };
    TRANSFER_EVENTS.with(|events| {
        events.borrow_mut().push(event);
    });

    Ok(true)
}

// Update to change the owner of the canister
#[update]
fn change_owner(new_owner: Principal) -> Result<(), TransferError> {
    if !is_owner() {
        return Err(TransferError::Unauthorized);
    }

    OWNER.with(|owner| {
        *owner.borrow_mut() = new_owner;
    });

    ic_cdk::println!("Owner changed to: {:?}", new_owner);
    Ok(())
}

// Query to get the transfer history
#[query]
fn get_transfer_history() -> Vec<TransferEvent> {
    TRANSFER_EVENTS.with(|events| events.borrow().clone())
}

// Utility function to check if the caller is the owner
fn is_owner() -> bool {
    let caller = caller();
    OWNER.with(|owner| *owner.borrow() == caller)
}

// Utility function to get the current time (in nanoseconds)
fn get_time() -> u64 {
    ic_cdk::api::time()
}

// // Main function (not used in canisters)
// fn main() {}

// Include tests if running in test mode

#[cfg(test)]
mod test_utils {
    use candid::Principal;
    use std::cell::RefCell;

    thread_local! {
        static MOCK_CALLER: RefCell<Principal> = RefCell::new(Principal::anonymous());
        static MOCK_TIME: RefCell<u64> = RefCell::new(0);
    }

    pub fn set_caller(principal: Principal) {
        MOCK_CALLER.with(|caller| *caller.borrow_mut() = principal);
    }

    // pub fn get_caller() -> Principal {
    //     MOCK_CALLER.with(|caller| *caller.borrow())
    // }

    // pub fn set_time(time: u64) {
    //     MOCK_TIME.with(|t| *t.borrow_mut() = time);
    // }

    // pub fn get_time() -> u64 {
    //     MOCK_TIME.with(|t| *t.borrow())
    // }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    #[test]
    fn test_create_wallet() {
        reset_state();
        let principal = Principal::anonymous();
        test_utils::set_caller(principal);
        let wallets = WALLETS.with(|wallets| wallets.borrow().clone());
        assert!(!wallets.contains_key(&principal));

        assert!(create_wallet().is_ok());
        let wallets = WALLETS.with(|wallets| wallets.borrow().clone());
        assert!(wallets.contains_key(&principal));
    }

    // Additional tests...
}

// Function to reset the state for testing
#[cfg(test)]
fn reset_state() {
    TOKEN.with(|token| {
        *token.borrow_mut() = Token::new(
            "ICP Token".to_string(),
            "ICPT".to_string(),
            8,
            1_000_000_000_000_000_000,
        );
    });
    WALLETS.with(|wallets| wallets.borrow_mut().clear());
    TRANSFER_EVENTS.with(|events| events.borrow_mut().clear());
    OWNER.with(|owner| *owner.borrow_mut() = Principal::anonymous());
}
