mod bitcoin_api;
mod bitcoin_wallet;
mod custody_wallet;
mod ecdsa_api;
mod types;

use ic_cdk::api::management_canister::bitcoin::{
    BitcoinNetwork, GetUtxosResponse, MillisatoshiPerByte,
};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
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
    static CUSTODY_WALLET: RefCell<custody_wallet::CustodyWallet> = RefCell::default();
}
#[derive(Clone, Debug, candid::Deserialize, candid::CandidType)]
pub struct InitArguments {
    pub bitcoin_network: BitcoinNetwork,
    pub fiduciary_id: candid::Principal,
}

#[init]
pub fn init(args: InitArguments) {

    let key = String::from(match args.bitcoin_network {
        // For local development, we use a special test key with dfx.
        BitcoinNetwork::Regtest => "dfx_test_key",
        // On the IC we're using a test ECDSA key.
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
    });

    NETWORK.with(|n| 
        n.set(args.bitcoin_network)
    );

    KEY_NAME.with(|key_name| {
        key_name.replace(key);
    });

    FIDUCIARY_ID.with(|id| {
        id.replace(Some(args.fiduciary_id));
    });
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
pub async fn create_wallet() {
    let network = NETWORK.with(|n| n.get());
    let key = KEY_NAME.with(|kn| kn.borrow().to_string());
    let fiduciary_id = FIDUCIARY_ID.with(|id| id.borrow().clone().unwrap());
    let wallet = custody_wallet::new(network, key, fiduciary_id).await;
    CUSTODY_WALLET.with(|custody_wallet| {
        custody_wallet.replace(wallet);
    });
}

#[query]
pub async fn get_address() -> String {
    CUSTODY_WALLET.with(|w| w.borrow().address.to_string())
}

/// Sends the given amount of bitcoin from this canister to the given address.
/// Returns the transaction ID.
#[update]
pub async fn send(request: types::SendRequest) -> String {
    let wallet = CUSTODY_WALLET.with(|w| w.borrow().clone());
    custody_wallet::send(&wallet, request.destination_address, request.amount_in_satoshi).await.to_string()
}

#[pre_upgrade]
fn pre_upgrade() {
    let network = NETWORK.with(|n| n.get());
    ic_cdk::storage::stable_save((network,)).expect("Saving network to stable store must succeed.");
}

#[post_upgrade]
async fn post_upgrade() {
    let bitcoin_network = ic_cdk::storage::stable_restore::<(BitcoinNetwork,)>()
        .expect("Failed to read network from stable memory.")
        .0;
    let fiduciary_id = FIDUCIARY_ID.with(|id| id.borrow().clone().unwrap());

    init({
        InitArguments {
            bitcoin_network,
            fiduciary_id,
        }});
}
