
# Token Wallet on the ICP
This repository contains a basic token system built on the Internet Computer (IC). The system supports functionalities for creating wallets, minting tokens, transferring tokens between wallets, and burning tokens.




## Prerequisites

1. **DFINITY Canister SDK (dfx)**

The DFINITY Canister SDK (`dfx`) should be installed. If it’s not installed, follow the setup guide here: [DFINITY SDK Setup](https://sdk.dfinity.org/docs/quickstart/local-quickstart.html)

2. **Clone the Repository**

3. **Navigate to the Project**
```bash
  cd <project_directory>
```

4. **Deploy the Canister**
```bash
  dfx deploy
```






## Usage Guide
**Wallet Creation** 

To set up a new wallet, execute the create_wallet method. This action associates a new wallet with your principal:

```bash
  dfx canister call icp_token create_wallet
```
**Token Minting** 

Minting tokens is restricted to the canister owner. Use the following command to mint tokens:

```bash
  dfx canister call icp_token mint '(principal "<recipient_principal>", <amount>)'
```
**Token Transfer** 

To transfer tokens to another wallet:

```bash
  dfx canister call icp_token transfer '(principal "<recipient_principal>", <amount>)'
```
**Token Burning** 

To burn tokens from your wallet, use:

```bash
  dfx canister call icp_token burn <amount>
```
**Balance Inquiry** 

To check the balance of a particular wallet:

```bash
  dfx canister call icp_token get_balance '(principal "<principal_id>")'
```
**Transfer History** 

To review the history of transfers:

```bash
  ddfx canister call icp_token get_transfer_history
```




## Error Management
The canister handles various errors including:

- Insufficient funds for transfers or burns

- Unauthorized minting or ownership changes

- Invalid transfer amounts

- Overflow errors for excessive amounts
## Testing

To execute tests, follow these steps:

- Ensure you are in the project directory.
- Run:
```bash
  cargo test
```

This command will run all unit tests defined in the test suite.