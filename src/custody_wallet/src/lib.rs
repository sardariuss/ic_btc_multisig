mod bitcoin_api;
mod custody_wallet;
mod ecdsa_api;
mod types;

use ic_cdk::api::management_canister::bitcoin::{
    BitcoinNetwork, GetUtxosResponse, MillisatoshiPerByte,
};
use ic_cdk::api;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, update, query};
use std::cell::{Cell, RefCell};

thread_local! {
    // The bitcoin network to connect to.
    //
    // When developing locally this should be `Regtest`.
    // When deploying to the IC this should be `Testnet`.
    // `Mainnet` is currently unsupported.
    static NETWORK: Cell<BitcoinNetwork> = Cell::new(BitcoinNetwork::Testnet);

    // The ECDSA key name.
    static KEY_NAME: RefCell<String> = RefCell::new(String::from(""));

    // The fiduciary canister.
    static FIDUCIARY_ID: RefCell<Option<candid::Principal>> = RefCell::new(None);

    // The custody wallet.
    static CUSTODY_WALLET: RefCell<custody_wallet::CustodyData> = RefCell::default();
}

#[derive(Clone, Debug, candid::Deserialize, candid::CandidType)]
pub struct InitArguments {
    pub bitcoin_network: BitcoinNetwork,
    pub fiduciary_id: candid::Principal,
}

#[init]
pub fn init(args: InitArguments) {

    let key = get_key_name(args.bitcoin_network);

    let custody_wallet = custody_wallet::CustodyData::new(
        args.bitcoin_network,
        key.clone(),
        args.fiduciary_id.clone()
    );

    NETWORK.with(|n| 
        n.set(args.bitcoin_network)
    );

    KEY_NAME.with(|key_name| {
        key_name.replace(key);
    });

    FIDUCIARY_ID.with(|id| {
        id.replace(Some(args.fiduciary_id));
    });

    CUSTODY_WALLET.with(|wallet| {
        wallet.replace(custody_wallet);
    });
}

#[query]
pub async fn get_network() -> BitcoinNetwork {
    NETWORK.with(|n| n.get())
}

/// Returns the balance of the given bitcoin address.
#[update]
pub async fn get_balance(address: String) -> u64 {
    let network = NETWORK.with(|n| n.get());
    bitcoin_api::get_balance(network, address).await
}

/// Returns the UTXOs of the given bitcoin address.
#[update]
pub async fn get_utxos(address: String) -> GetUtxosResponse {
    let network = NETWORK.with(|n| n.get());
    bitcoin_api::get_utxos(network, address).await
}

/// Returns the 100 fee percentiles measured in millisatoshi/byte.
/// Percentiles are computed from the last 10,000 transactions (if available).
#[update]
pub async fn get_current_fee_percentiles() -> Vec<MillisatoshiPerByte> {
    let network = NETWORK.with(|n| n.get());
    bitcoin_api::get_current_fee_percentiles(network).await
}

#[update]
pub async fn get_wallet_address() -> String {
    let principal = &api::caller();
    let mut custody_wallet = CUSTODY_WALLET.with(|w| w.borrow().clone());
    let address = custody_wallet::get_or_create_wallet(&mut custody_wallet, principal.clone()).await;
    CUSTODY_WALLET.with(|wallet| {
        wallet.replace(custody_wallet);
    });
    address.to_string()
}

/// Sends the given amount of bitcoin from the principal's wallet to the given address.
/// Returns the transaction ID.
#[update]
pub async fn wallet_send(request: types::SendRequest) -> String {
    let principal = &api::caller();
    let wallet = CUSTODY_WALLET.with(|w| w.borrow().clone());
    custody_wallet::send(
        &wallet, 
        principal.clone(), 
        request.destination_address, 
        request.amount_in_satoshi)
    .await.to_string()
}

#[query]
pub async fn get_ecdsa_key_name(bitcoin_network: BitcoinNetwork) -> String {
    String::from(match bitcoin_network {
        // For local development, we use a special test key with dfx.
        BitcoinNetwork::Regtest => "dfx_test_key",
        // On the IC we're using a test ECDSA key.
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
    })
}

fn get_key_name(bitcoin_network: BitcoinNetwork) -> String {
    String::from(match bitcoin_network {
        // For local development, we use a special test key with dfx.
        BitcoinNetwork::Regtest => "dfx_test_key",
        // On the IC we're using a test ECDSA key.
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
    })
}

#[pre_upgrade]
fn pre_upgrade() {
    let bitcoin_network = NETWORK.with(|n| n.get());
    let fiduciary_id = FIDUCIARY_ID.with(|id| id.borrow().clone().unwrap());
    ic_cdk::storage::stable_save((bitcoin_network, fiduciary_id,)).expect("Saving bitcoin network and fiduciary ID to stable store must succeed.");
}

#[post_upgrade]
async fn post_upgrade() {
    let (bitcoin_network, fiduciary_id) = ic_cdk::storage::stable_restore::<(BitcoinNetwork, candid::Principal)>()
        .expect("Failed to read bitcoin network and fiduciary ID from stable memory.");

    init({
        InitArguments {
            bitcoin_network,
            fiduciary_id,
        }});
}
