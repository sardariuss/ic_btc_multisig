use multisig_common::{
    common,
    types::{BitcoinNetwork, SendRequest, RawTransactionInfo},
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
    static CUSTODY_WALLET: RefCell<common::CustodyData> = RefCell::default();
}

#[derive(Clone, Debug, candid::Deserialize, candid::CandidType)]
pub struct InitArguments {
    pub bitcoin_network: BitcoinNetwork,
    pub fiduciary_id: candid::Principal,
}

#[init]
pub fn init(args: InitArguments) {

    let key = get_key_name(args.bitcoin_network);

    let custody_wallet = common::CustodyData::new(
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

#[query]
pub async fn get_ecdsa_key_name(bitcoin_network: BitcoinNetwork) -> String {
    String::from(match bitcoin_network {
        // For local development, we use a special test key with dfx.
        BitcoinNetwork::Regtest => "dfx_test_key",
        // On the IC we're using a test ECDSA key.
        BitcoinNetwork::Mainnet | BitcoinNetwork::Testnet => "test_key_1",
    })
}

/// Returns the balance of the given bitcoin address.
#[update]
pub async fn get_balance(address: String) -> u64 {
    let network = NETWORK.with(|n| n.get());
    common::get_balance(network, address).await
}

#[update]
pub async fn get_wallet_address() -> String {
    let principal = &api::caller();
    let mut custody_wallet = CUSTODY_WALLET.with(|w| w.borrow().clone());
    let address = common::get_or_create_wallet(&mut custody_wallet, principal.clone()).await;
    CUSTODY_WALLET.with(|wallet| {
        wallet.replace(custody_wallet);
    });
    address.to_string()
}

#[update]
pub async fn init_send_request(send_request: SendRequest) -> RawTransactionInfo {
    
    let principal = &api::caller();
    let custody_data = CUSTODY_WALLET.with(|w| w.borrow().clone());        

    // Build the transaction.
    let mut transaction_info = common::build_unsigned_transaction(
        &custody_data,
        principal.clone(),
        send_request.destination_address, 
        send_request.amount_in_satoshi)
    .await;

    let bitcoin_network = NETWORK.with(|n| n.get());
    let key_name = get_key_name(bitcoin_network);

    // Insert the first signature.
    transaction_info = common::sign_transaction(
        &transaction_info,
        &key_name,
        &vec![principal.as_slice().to_vec()],
        common::MultisigIndex::First)
    .await;

    // Return the raw transaction info.
    transaction_info.to_raw()
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
