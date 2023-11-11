//! A demo of a very bare-bones bitcoin "wallet".
//!
//! The wallet here showcases how bitcoin addresses can be be computed
//! and how bitcoin transactions can be signed. It is missing several
//! pieces that any production-grade wallet would have, including:
//!
//! * Support for address types that aren't P2PKH.
//! * Caching spent UTXOs so that they are not reused in future transactions.
//! * Option to set the fee.
use crate::bitcoin_api;
use bitcoin::Sequence;
use bitcoin::absolute::LockTime;
use bitcoin::hex::DisplayHex;
use bitcoin::secp256k1::Keypair;
//use bitcoin::hashes::hex::ToHex;
//use bitcoin::util::psbt::serialize::Serialize;
use bitcoin::{
    blockdata::witness::Witness,
    hashes::Hash,
    //network::constants::Network,
    Address, EcdsaSighashType, OutPoint, Script, Transaction, TxIn, TxOut, Txid,
    consensus,
    ScriptBuf,
    network::Network,
    key::Secp256k1,
    amount::Amount,
    sighash,
    ecdsa
};
use ic_cdk::api::management_canister::bitcoin::{MillisatoshiPerByte, BitcoinNetwork, Satoshi, Utxo};
use ic_cdk::print;
use bitcoin::key::secp256k1::{Message, SecretKey};
use std::str::FromStr;

const SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;

pub fn create_multisig_2x2_witness_script() -> ScriptBuf {
    // Fetch the public key of the given derivation path.
    let (kp1, kp2) = generate_2_pairs_of_keys();

    // Create a 2-of-2 multisig witness script.
    bitcoin::blockdata::script::Builder::new()
        .push_int(2)
        .push_slice(&kp1.public_key().serialize())
        .push_slice(&kp2.public_key().serialize())
        .push_int(2)
        .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG)
        .into_script()
}

pub fn get_p2wsh_multisig_2x2_address(
    //_network: BitcoinNetwork,
    //_key_name: String,
    //_derivation_path: Vec<Vec<u8>>,
) -> String {
    let witness_script = create_multisig_2x2_witness_script();

//    let script_pub_key = bitcoin::blockdata::script::Builder::new()
//        .push_int(0)
//        .push_slice(witness_script.script_hash().as_inner().as_slice())
//        .into_script();

    let script_pub_key = ScriptBuf::new_p2wsh(&witness_script.wscript_hash());

    let address = bitcoin::Address::from_script(&script_pub_key, Network::Testnet).unwrap();

    address.to_string()
}

pub fn generate_2_pairs_of_keys() -> (Keypair, Keypair) {
    let secp = bitcoin::key::Secp256k1::new();
    let pair_1 = Keypair::from_seckey_str(&secp, "03d9cd11d73f84fcd33308143eaffa3e9b3b353f85ec5e608e5ce81d0eecb5aa").unwrap();
    let pair_2 = Keypair::from_seckey_str(&secp, "c0395f033eaace8cb17cc3249c56f706a5490daeeebc0476ae18a73c21d8c63f").unwrap();
    return (pair_1, pair_2);
}

pub fn generate_2_pairs_of_keys_string() -> ((String, String), (String, String)) {
    let (kp1, kp2) = generate_2_pairs_of_keys();
    ((kp1.display_secret().to_string(), hex::encode(kp1.public_key().serialize())), ((kp2.display_secret().to_string()), hex::encode(kp2.public_key().serialize())))
}

pub async fn script_sig(
    network: BitcoinNetwork,
    dst_address: String,
    amount: Satoshi,
) -> String {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = bitcoin_api::get_current_fee_percentiles(network).await;

    let fee_per_byte = if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        2000
    } else {
        // Choose the 50th percentile for sending fees.
        fee_percentiles[50]
    };

    // Fetch our public key, P2PKH address, and UTXOs.
    let own_address = get_p2wsh_multisig_2x2_address();

    print("Fetching UTXOs...");
    // Note that pagination may have to be used to get all UTXOs for the given address.
    // For the sake of simplicity, it is assumed here that the `utxo` field in the response
    // contains all UTXOs.
    let own_utxos = bitcoin_api::get_utxos(network, own_address.clone())
        .await
        .utxos;

    let own_address = Address::from_str(&own_address).unwrap().assume_checked();
    let dst_address = Address::from_str(&dst_address).unwrap().assume_checked();

    // Build the transaction that sends `amount` to the destination address.
    let built_transaction_output = build_transaction(
        &own_address,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    let witness_script = create_multisig_2x2_witness_script();

    let mut cache = sighash::SighashCache::new(&built_transaction_output.transaction);

    let sighash = cache
        .p2wsh_signature_hash(0, &witness_script, Amount::from_sat(built_transaction_output.output_amount_satoshi), EcdsaSighashType::All).expect("failed to compute sighash");
    return sighash.to_byte_array().as_hex().to_string();
}

pub async fn transaction_from_multisig_2x2(
    network: BitcoinNetwork,
    dst_address: String,
    amount: Satoshi,
) -> String {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = bitcoin_api::get_current_fee_percentiles(network).await;

    let fee_per_byte = if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        2000
    } else {
        // Choose the 50th percentile for sending fees.
        fee_percentiles[50]
    };

    // Fetch our public key, P2PKH address, and UTXOs.
    let own_address = get_p2wsh_multisig_2x2_address();

    print("Fetching UTXOs...");
    // Note that pagination may have to be used to get all UTXOs for the given address.
    // For the sake of simplicity, it is assumed here that the `utxo` field in the response
    // contains all UTXOs.
    let own_utxos = bitcoin_api::get_utxos(network, own_address.clone())
        .await
        .utxos;

    let own_address = Address::from_str(&own_address).unwrap().assume_checked();
    let dst_address = Address::from_str(&dst_address).unwrap().assume_checked();

    // Build the transaction that sends `amount` to the destination address.
    let built_transaction_output = build_transaction(
        &own_address,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    //let tx_bytes = transaction.serialize();
    //print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign the transaction.
    let signed_transaction = sign_transaction(
        built_transaction_output.transaction,
        sign_with_ecdsa,
        built_transaction_output.output_amount_satoshi,
    );

    //let signed_transaction_bytes = signed_transaction.serialize();
    let signed_transaction_bytes = consensus::serialize(&signed_transaction);
    print(&format!(
        "Signed transaction: {}",
        hex::encode(&signed_transaction_bytes)
    ));

    hex::encode(&signed_transaction_bytes)
}

/// Sends a transaction to the network that transfers the given amount to the
/// given destination, where the source of the funds is the canister itself
/// at the given derivation path.
pub async fn send_from_multisig_2x2(
    network: BitcoinNetwork,
    dst_address: String,
    amount: Satoshi,
) -> Txid {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = bitcoin_api::get_current_fee_percentiles(network).await;

    let fee_per_byte = if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        2000
    } else {
        // Choose the 50th percentile for sending fees.
        fee_percentiles[50]
    };

    // Fetch our public key, P2PKH address, and UTXOs.
    let own_address = get_p2wsh_multisig_2x2_address();

    print("Fetching UTXOs...");
    // Note that pagination may have to be used to get all UTXOs for the given address.
    // For the sake of simplicity, it is assumed here that the `utxo` field in the response
    // contains all UTXOs.
    let own_utxos = bitcoin_api::get_utxos(network, own_address.clone())
        .await
        .utxos;

    let own_address = Address::from_str(&own_address).unwrap().assume_checked();
    let dst_address = Address::from_str(&dst_address).unwrap().assume_checked();

    // Build the transaction that sends `amount` to the destination address.
    let built_transaction_output = build_transaction(
        &own_address,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    //let tx_bytes = transaction.serialize();
    //print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign the transaction.
    let signed_transaction = sign_transaction(
        built_transaction_output.transaction,
        sign_with_ecdsa,
        built_transaction_output.output_amount_satoshi,
    );

    let signed_transaction_bytes = consensus::serialize(&signed_transaction);
    print(&format!(
        "Signed transaction: {}",
        hex::encode(&signed_transaction_bytes)
    ));

    print("Sending transaction...");
    bitcoin_api::send_transaction(network, signed_transaction_bytes).await;
    print("Done");

    signed_transaction.txid()
}

// Builds a transaction to send the given `amount` of satoshis to the
// destination address.
async fn build_transaction(
    own_address: &Address,
    own_utxos: &[Utxo],
    dst_address: &Address,
    amount: Satoshi,
    _fee_per_byte: MillisatoshiPerByte,
) -> BuiltTransactionOutput {
    // We have a chicken-and-egg problem where we need to know the length
    // of the transaction in order to compute its proper fee, but we need
    // to know the proper fee in order to figure out the inputs needed for
    // the transaction.
    //
    // We solve this problem iteratively. We start with a fee of zero, build
    // and sign a transaction, see what its size is, and then update the fee,
    // rebuild the transaction, until the fee is set to the correct amount.
    print("Building transaction...");
    let total_fee = 10_000;
    //loop {
        let transaction =
            build_transaction_with_fee(own_utxos, own_address, dst_address, amount, total_fee)
                .expect("Error building transaction.");

        // Sign the transaction. In this case, we only care about the size
        // of the signed transaction, so we use a mock signer here for efficiency.
//        let signed_transaction = sign_transaction(
//            own_address,
//            transaction.clone(),
//            mock_signer
//        );

        //let signed_tx_bytes_len = signed_transaction.serialize().len() as u64;

        //if (signed_tx_bytes_len * fee_per_byte) / 1000 == total_fee {
            print(&format!("Transaction built with fee {}.", total_fee));
            return transaction;
        //} else {
            //total_fee = (signed_tx_bytes_len * fee_per_byte) / 1000;
        //}
    //}
}

pub struct BuiltTransactionOutput {
    pub transaction: Transaction,
    pub output_amount_satoshi: u64,
}

fn build_transaction_with_fee(
    own_utxos: &[Utxo],
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    fee: u64,
) -> Result<BuiltTransactionOutput, String> {

    // Assume that any amount below this threshold is dust.
    const DUST_THRESHOLD: u64 = 1_000;

    // Select which UTXOs to spend. We naively spend the oldest available UTXOs,
    // even if they were previously spent in a transaction. This isn't a
    // problem as long as at most one transaction is created per block and
    // we're using min_confirmations of 1.
    let mut utxos_to_spend = vec![];
    let mut total_spent = 0;
    for utxo in own_utxos.iter().rev() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        if total_spent >= amount + fee {
            // We have enough inputs to cover the amount we want to spend.
            break;
        }
    }

    if total_spent < amount + fee {
        return Err(format!(
            "Insufficient balance: {}, trying to transfer {} satoshi with fee {}",
            total_spent, amount, fee
        ));
    }

    let inputs: Vec<TxIn> = utxos_to_spend
        .into_iter()
        .map(|utxo| TxIn {
            previous_output: OutPoint {
                txid: Txid::from_raw_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: Sequence::MAX, // 0xffffffff,
            witness: Witness::new(),
            script_sig: ScriptBuf::new(),
        })
        .collect();

    let mut outputs = vec![TxOut {
        script_pubkey: dst_address.script_pubkey(),
        value: Amount::from_sat(amount),
    }];

    let remaining_amount = total_spent - amount - fee;

    if remaining_amount >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: Amount::from_sat(remaining_amount),
        });
    }

    Ok(BuiltTransactionOutput{
        transaction: Transaction {
        input: inputs,
        output: outputs,
        lock_time: LockTime::ZERO, // @todo: verify
        version: bitcoin::blockdata::transaction::Version::ONE, // @todo: verify
        },
        output_amount_satoshi: total_spent,
    })
}

// Sign a bitcoin transaction.
//
// IMPORTANT: This method is for demonstration purposes only and it only
// supports signing transactions if:
//
// 1. All the inputs are referencing outpoints that are owned by `own_address`.
// 2. `own_address` is a P2PKH address.
fn sign_transaction<SignFun>(
    mut transaction: Transaction,
    signer: SignFun,
    amount_satoshi: u64,
) -> Transaction
where
    SignFun: Fn(&SecretKey, Vec<u8>) -> Vec<u8>,
{
//    // Verify that our own address is P2WSH.
//    assert_eq!(
//        own_address.address_type(),
//        Some(AddressType::P2wsh),
//        "This example supports signing p2wsh addresses only."
//    );

    let witness_script = create_multisig_2x2_witness_script();

    let txclone = transaction.clone();
    let mut cache = sighash::SighashCache::new(&txclone);

    for (index, input) in transaction.input.iter_mut().enumerate() {
        // Clear any previous witness
        input.witness.clear();

        let sighash = cache
            .p2wsh_signature_hash(index, &witness_script, Amount::from_sat(amount_satoshi), EcdsaSighashType::All).expect("failed to compute sighash");

        let (kp1, kp2) = generate_2_pairs_of_keys();
        let mut signature_der_1 = signer(&kp1.secret_key(), sighash.to_byte_array().to_vec());
        signature_der_1.push(SIG_HASH_TYPE.to_u32() as u8);
        let mut signature_der_2 = signer(&kp2.secret_key(), sighash.to_byte_array().to_vec());
        signature_der_2.push(SIG_HASH_TYPE.to_u32() as u8);

        input.witness.push(vec![]);  // Placeholder for scriptSig
        input.witness.push(signature_der_1);
        input.witness.push(signature_der_2);
        input.witness.push(witness_script.clone().into_bytes());
    }

    transaction
}

fn mock_signer(
    _sk: &SecretKey,
    _message_hash: Vec<u8>,
) -> Vec<u8> {
    vec![255; 64]
}

fn sign_with_ecdsa(
    sk: &SecretKey,
    message_hash: Vec<u8>,
) -> Vec<u8> {
    let secp = Secp256k1::new();
    let signature = Secp256k1::sign_ecdsa(&secp, &Message::from_digest_slice(&message_hash).unwrap(), &sk);
    signature.serialize_der().to_vec()
}
