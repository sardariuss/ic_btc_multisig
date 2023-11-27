use multisig_common::{
    common, 
    types::{BitcoinNetwork, RawTransactionInfo},
};
use ic_cdk::api;
use ic_cdk_macros::{query, update, init};

#[init]
pub fn init() {
}

#[query]
pub async fn get_ecdsa_key_name(bitcoin_network: BitcoinNetwork) -> String {
    get_key_name(bitcoin_network)
}

#[update]
pub async fn public_key(network: BitcoinNetwork, derivation_path: Vec<Vec<u8>>) -> Vec<u8> {
    common::ecdsa_public_key(
        get_key_name(network),
        derivation_path)
    .await
}

#[update]
pub async fn finalize_send_request(bitcoin_network: BitcoinNetwork, raw_transaction_info: RawTransactionInfo) -> String {
    
    let principal = &api::caller();
    let key_name = get_key_name(bitcoin_network);
    
    // Get the transaction info from the raw one.
    let mut transaction_info = common::TransactionInfo::from_raw(raw_transaction_info);

    // Insert the second (and last) signature.
    transaction_info = common::sign_transaction(
        &transaction_info,
        &key_name,
        &vec![principal.as_slice().to_vec()],
        common::MultisigIndex::Last)
        .await;

    // Send the transaction.
    common::send_transaction(bitcoin_network, &transaction_info).await;

    // Return the transaction id.
    transaction_info.transaction().txid().to_string()
}

fn get_key_name(bitcoin_network: BitcoinNetwork) -> String {
    String::from(match bitcoin_network {
        // For local development, we use a special test key with dfx.
        BitcoinNetwork::Regtest => "dfx_test_key",
        // On the IC we're using key_1 for testnet and mainnet.
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "key_1",
    })
}
