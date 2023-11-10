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
use bitcoin::util::psbt::serialize::Serialize;
use bitcoin::{
    blockdata::witness::Witness,
    hashes::Hash,
    network::constants::Network,
    Address, AddressType, EcdsaSighashType, OutPoint, Script, Transaction, TxIn, TxOut, Txid,
    secp256k1::Secp256k1,
};
use ic_cdk::api::management_canister::bitcoin::{MillisatoshiPerByte, BitcoinNetwork, Satoshi, Utxo};
use ic_cdk::print;
use secp256k1::{Message, SecretKey, PublicKey};
use std::str::FromStr;
use secp256k1::rand::SeedableRng;
use secp256k1::rand::rngs;

const SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;

pub fn create_multisig_2x2_witness_script() -> Script {
    // Fetch the public key of the given derivation path.
    let ((_, pk1), (_, pk2)) = generate_2_pairs_of_keys();

    // Create a 2-of-2 multisig witness script.
    bitcoin::blockdata::script::Builder::new()
        .push_int(2)
        .push_slice(&pk1.serialize())
        .push_slice(&pk2.serialize())
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

    let script_pub_key = Script::new_v0_p2wsh(&witness_script.wscript_hash());

    let address = bitcoin::Address::from_script(&script_pub_key, Network::Testnet).unwrap();

    address.to_string()
}

pub fn generate_2_pairs_of_keys() -> ((SecretKey, PublicKey), (SecretKey, PublicKey)) {
    let secp = Secp256k1::new();
    let mut rng = rngs::SmallRng::seed_from_u64(0);
    (Secp256k1::generate_keypair(&secp, &mut rng), Secp256k1::generate_keypair(&secp, &mut rng))
}

pub fn generate_2_public_keys() -> (String, String) {
    let secp = Secp256k1::new();
    let mut rng = rngs::SmallRng::seed_from_u64(0);
    let (_, pk1) = Secp256k1::generate_keypair(&secp, &mut rng);
    let (_, pk2) = Secp256k1::generate_keypair(&secp, &mut rng);
    (hex::encode(pk1.serialize()), hex::encode(pk2.serialize()))
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

    let own_address = Address::from_str(&own_address).unwrap();
    let dst_address = Address::from_str(&dst_address).unwrap();

    // Build the transaction that sends `amount` to the destination address.
    let transaction = build_transaction(
        &own_address,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    let tx_bytes = transaction.serialize();
    print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign the transaction.
    let signed_transaction = sign_transaction(
        &own_address,
        transaction,
        sign_with_ecdsa,
    );

    let signed_transaction_bytes = signed_transaction.serialize();
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

    let own_address = Address::from_str(&own_address).unwrap();
    let dst_address = Address::from_str(&dst_address).unwrap();

    // Build the transaction that sends `amount` to the destination address.
    let transaction = build_transaction(
        &own_address,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    let tx_bytes = transaction.serialize();
    print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign the transaction.
    let signed_transaction = sign_transaction(
        &own_address,
        transaction,
        sign_with_ecdsa,
    );

    let signed_transaction_bytes = signed_transaction.serialize();
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
) -> Transaction {
    // We have a chicken-and-egg problem where we need to know the length
    // of the transaction in order to compute its proper fee, but we need
    // to know the proper fee in order to figure out the inputs needed for
    // the transaction.
    //
    // We solve this problem iteratively. We start with a fee of zero, build
    // and sign a transaction, see what its size is, and then update the fee,
    // rebuild the transaction, until the fee is set to the correct amount.
    print("Building transaction...");
    let total_fee = 100_000;
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

fn build_transaction_with_fee(
    own_utxos: &[Utxo],
    own_address: &Address,
    dst_address: &Address,
    amount: u64,
    fee: u64,
) -> Result<Transaction, String> {

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
                txid: Txid::from_hash(Hash::from_slice(&utxo.outpoint.txid).unwrap()),
                vout: utxo.outpoint.vout,
            },
            sequence: 0xffffffff,
            witness: Witness::new(),
            script_sig: Script::new(),
        })
        .collect();

    let mut outputs = vec![TxOut {
        script_pubkey: dst_address.script_pubkey(),
        value: amount,
    }];

    let remaining_amount = total_spent - amount - fee;

    if remaining_amount >= DUST_THRESHOLD {
        outputs.push(TxOut {
            script_pubkey: own_address.script_pubkey(),
            value: remaining_amount,
        });
    }

    Ok(Transaction {
        input: inputs,
        output: outputs,
        lock_time: 0,
        version: 1,
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
    own_address: &Address,
    mut transaction: Transaction,
    signer: SignFun,
) -> Transaction
where
    SignFun: Fn(&SecretKey, Vec<u8>) -> Vec<u8>,
{
    // Verify that our own address is P2WSH.
    assert_eq!(
        own_address.address_type(),
        Some(AddressType::P2wsh),
        "This example supports signing p2wsh addresses only."
    );

    let witness_script = create_multisig_2x2_witness_script();

    let txclone: Transaction = transaction.clone();
    for (index, input) in transaction.input.iter_mut().enumerate() {
        // Clear any previous witness
        input.witness.clear();

        let sighash =
            txclone.signature_hash(index, &witness_script.clone(), SIG_HASH_TYPE.to_u32());

        let ((sk1, _), (sk2, _)) = generate_2_pairs_of_keys();
        //let mut signatures = vec![];
        //signatures.append(&mut signer(&sk1, sighash.to_vec()));
        //signatures.push(SIG_HASH_TYPE.to_u32() as u8);
        //signatures.append(&mut signer(&sk2, sighash.to_vec()));
        //signatures.push(SIG_HASH_TYPE.to_u32() as u8);
        let mut signature_der_1 = signer(&sk1, sighash.to_vec());
        signature_der_1.push(SIG_HASH_TYPE.to_u32() as u8);
        let mut signature_der_2 = signer(&sk2, sighash.to_vec());
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
    let signature = Secp256k1::sign_ecdsa(&secp, &Message::from_slice(&message_hash).unwrap(), &sk);
    signature.serialize_der().to_vec()
}

