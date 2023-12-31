# Multi-subnet wallet on the Internet Computer

## ⚠️ Disclaimer

**USE AT YOUR OWN RISK!**

The following code is provided as-is and has not undergone deep testing or auditing. It is strongly advised not to use this code for transferring real bitcoin funds on the mainnet. The authors and contributors disclaim any responsibility for potential issues or losses resulting from the use of this code.

## 🚀 Deploy the smart contract on the [Internet Computer](https://internetcomputer.org/)

The contracts are already deployed on the IC for the Bitcoin testnet [here](https://o3zgr-waaaa-aaaap-abr2q-cai.raw.icp0.io/).  
To deploy it yourself on the IC, you can inspire yourself from the script _install-ic.sh_. However note that to install the fiduciary canister on the fiduciary subnet, the only present way to do so is to create the canister with the dfx ledger command:  
```dfx ledger create-canister --subnet-type=fiduciary --e8s=50000000 --ic <CONTROLLER>```

## 🔎 How does it work?

The Internet Computer can be used to create Bitcoin wallets, where the private key never exists physically, but signatures are generated by a secure multi-party computation among the nodes of a subnet. To make such wallet even more secure, we can create P2WSH multi-sig addresses that request signatures generated from different subnets, so that the node providers of these subnets would need to collude to steal the assets.

### Architecture

The multi-subnet wallet is composed of two backend canisters: the custody wallet canister and the fiduciary canister.  
The custody wallet is the main canister. It interfaces with the bitcoin API to get the balance, query the UTXOs and construct the transaction to send when withdrawing funds.  
The fiduciary canister is the second canister which jobs are to create to generate the second public key for the multi-sig address and insert the second signature then send the multi-sig transaction created by the custody wallet canister.
Both canisters use the code from src/multisig_common.

### Signatures

The canisters generate 2x2 multisig bitcoin P2WSH addresses based on two public keys:  
 - the first pk is generated by the custody wallet itself, directly calling the ecdsa_public_key method with the key name "test_key_1"
 - the second pk is generated by the fiduciary canister, which also calls the ecdsa_public_key method but with a different key name (hard-coded to "key_1")
The principal's caller is used for derivation path so that a unique bitcoin address and signature is derived for each user.

### Address creation flow

```mermaid
sequenceDiagram
Frontend->>Custody Wallet: get_bitcoin_address
Custody Wallet->>ECDSA API: ecdsa_public_key("test_key_1", principal)
Custody Wallet->>Fiduciary: public_key(principal)
Fiduciary->>ECDSA API: ecdsa_public_key("key_1", principal)
Note right of Custody Wallet: generate 2x2 P2WSH multisig address
Custody Wallet->>Frontend: address
```

When a new address is generated for a user, the custody wallet canister keeps in memory the witness script that has been used to generate the address for that user. When the user sends funds from that address, the associated script is used to generate sighashes which are signed by both canisters and added to the witness to form a valid transaction.  

The withdrawal process unfolds in two distinct stages. First the custody wallet canister creates the transaction and add the first signature by signing the sighash itself. Then the transaction is passed to the fiduciary canister which generates and adds the second signature and finally sends the transaction to the bitcoin network.

### Withdrawal flow

```mermaid
sequenceDiagram
Frontend->>Custody Wallet: init_send_request
Custody Wallet->>Bitcoin API: get_utxos(btc_address)
Bitcoin API-->>Custody Wallet: utxos
Custody Wallet->>Custody Wallet: build transaction
Custody Wallet->>ECDSA API: sign_with_ecdsa("test_key_1", principal, sighash)
ECDSA API-->>Custody Wallet: signature 1
Custody Wallet->>Custody Wallet: insert signature 1
Custody Wallet-->>Frontend: transaction
Frontend->>Fiduciary: finalize_send_request(transaction)
Fiduciary->>ECDSA API: sign_with_ecdsa("key_1", principal, sighash)
ECDSA API-->>Fiduciary: signature 2
Fiduciary->>Fiduciary: insert signature 2
Fiduciary->>Bitcoin API: send_transaction(transaction)
Fiduciary-->>Frontend: transaction identifier
```

## 📃 Notes

 - The fee to sign_with_ecdsa is set to 25 billions (contrary to the 10 billions found in the dfinity btc example) otherwise the ecdsa signature sometimes failed with the error "insufficient cycles".

## 🚧 Pending improvements

 - [ ] During pre-upgrade, save the CustodyData in stable memory to restore it after the upgrade
 - [ ] Add an estimation of the fee to send bitcoins in the UI
 - [ ] Allow the user to change the bitcoin network live
 - [ ] Allow each user to have multiple accounts (e.g. incremental suffix added to principal for the derivation path)
 - [ ] Ideally, the principal of the fiduciary canister shall be hard-coded in the custody wallet (instead of injected during the install)
 - [ ] Test on Bitcoin mainnet
 - [ ] Audit, black-hole, create verifiable wasm hash

## 🙏 Credits

 - The rust code is based on Dfinity's [bitcoin example](https://github.com/rust-bitcoin/rust-bitcoin/tree/master)
 - The welcome logo effects are from the [Vite-React-Motoko template](https://github.com/rvanasa/vite-react-motoko/tree/main)
 - The CSS title [glitch effect](https://codepen.io/aldrie/pen/PojGYLo)
