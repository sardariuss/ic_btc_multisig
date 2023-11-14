/// @todo
use crate::{bitcoin_api, ecdsa_api};
use bitcoin::Sequence;
use bitcoin::absolute::LockTime;
use bitcoin::address::NetworkChecked;
use secp256k1::PublicKey;
use bitcoin::{
    blockdata::witness::Witness,
    hashes::Hash,
    Address, EcdsaSighashType, OutPoint, Transaction, TxIn, TxOut, Txid,
    consensus,
    ScriptBuf,
    network::Network,
    amount::Amount,
    sighash,
};
use ic_cdk::api::management_canister::bitcoin::{MillisatoshiPerByte, BitcoinNetwork, Satoshi, Utxo};
use ic_cdk::{call, print};
use std::str::FromStr;

const SIG_HASH_TYPE: EcdsaSighashType = EcdsaSighashType::All;

fn match_network(bitcoin_network: BitcoinNetwork) -> Network {
    match bitcoin_network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

enum SigningMethod {
    Fake,
    Real,
}

#[derive(Clone)]
pub struct BuiltTransactionOutput {
    pub transaction: Transaction,
    pub input_amounts: Vec<Amount>,
}

#[derive(Clone)]
pub struct CustodyWallet {
    pub network: BitcoinNetwork,
    pub key_name: String,
    pub fiduciary_canister: candid::Principal,
    pub witness_script: ScriptBuf,
    pub address: Address<NetworkChecked>,
}

impl Default for CustodyWallet {
    fn default() -> Self {
        CustodyWallet {
            network: BitcoinNetwork::Regtest,
            key_name: String::from(""),
            fiduciary_canister: candid::Principal::anonymous(),
            witness_script: ScriptBuf::new(),
            address: Address::from_str("mopkf9Tud7qGd5nyT1qfvBMabYeemy92Pu").unwrap().assume_checked(), // @todo
        }
    }
}

pub async fn new(network: BitcoinNetwork, key_name: String, fiduciary_canister: candid::Principal) -> CustodyWallet {

    let pk1 = ecdsa_api::ecdsa_public_key(key_name.clone(), vec![], Option::None).await;
    
    let fiduciary_pk: Result<(Vec<u8>,), _> = call(
        fiduciary_canister,
        "public_key",
        (),
    )
    .await;

    let pk2 = fiduciary_pk.expect("Failed to obtain public key from fiduciary canister.").0;
  
    // Create a 2-of-2 multisig witness script.
    let witness_script = bitcoin::blockdata::script::Builder::new()
        .push_int(2)
        .push_slice(PublicKey::from_slice(pk1.as_slice()).unwrap().serialize())
        .push_slice(PublicKey::from_slice(pk2.as_slice()).unwrap().serialize())
        .push_int(2)
        .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG)
        .into_script();

    let script_pub_key = ScriptBuf::new_p2wsh(&witness_script.wscript_hash());

    let address = match bitcoin::Address::from_script(&script_pub_key, match_network(network)) {
        Ok(address) => {
           address
        }
        Err(error) => {
            panic!("Failed to generate bitcoin address from P2WSH script pubkey: {}", error);
        }
    };

    CustodyWallet {
        network,
        key_name,
        fiduciary_canister,
        witness_script,
        address,
    }
}

/// Sends a transaction to the network that transfers the given amount to the
/// given destination, where the source of the funds is the canister it
/// at the given derivation path.
pub async fn send(
    wallet: &CustodyWallet,
    dst_address: String,
    amount: Satoshi,
) -> Txid {
    // Get fee percentiles from previous transactions to estimate our own fee.
    let fee_percentiles = bitcoin_api::get_current_fee_percentiles(wallet.network).await;

    let fee_per_byte = if fee_percentiles.is_empty() {
        // There are no fee percentiles. This case can only happen on a regtest
        // network where there are no non-coinbase transactions. In this case,
        // we use a default of 2000 millisatoshis/byte (i.e. 2 satoshi/byte)
        2000
    } else {
        // Choose the 50th percentile for sending fees.
        fee_percentiles[50]
    };

    print("Fetching UTXOs...");
    // Note that pagination may have to be used to get all UTXOs for the given address.
    // For the sake of simplicity, it is assumed here that the `utxo` field in the response
    // contains all UTXOs.
    let own_utxos = bitcoin_api::get_utxos(wallet.network, wallet.address.to_string())
        .await
        .utxos;

    let dst_address = Address::from_str(&dst_address).unwrap().assume_checked();

    // Build the transaction that sends `amount` to the destination address.
    let built_transaction_output = build_transaction(
        wallet,
        &own_utxos,
        &dst_address,
        amount,
        fee_per_byte,
    )
    .await;

    let tx_bytes = consensus::serialize(&built_transaction_output.transaction);
    print(&format!("Transaction to sign: {}", hex::encode(tx_bytes)));

    // Sign the transaction.
    let signed_transaction = sign_transaction(
        wallet,
        built_transaction_output,
        SigningMethod::Real,
    ).await;

    let signed_transaction_bytes = consensus::serialize(&signed_transaction);
    print(&format!(
        "Signed transaction: {}",
        hex::encode(&signed_transaction_bytes)
    ));

    print("Sending transaction...");
    bitcoin_api::send_transaction(wallet.network, signed_transaction_bytes).await;
    print("Done");

    signed_transaction.txid()
}

// Builds a transaction to send the given `amount` of satoshis to the
// destination address.
async fn build_transaction(
    wallet: &CustodyWallet,
    own_utxos: &[Utxo],
    dst_address: &Address,
    amount: Satoshi,
    fee_per_byte: MillisatoshiPerByte,
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
    let mut total_fee = 0;
    loop {
        let built_transaction =
            build_transaction_with_fee(wallet, own_utxos, dst_address, amount, total_fee)
                .expect("Error building transaction.");

        // Sign the transaction. In this case, we only care about the size
        // of the signed transaction, so we use a mock signer here for efficiency.
        let signed_transaction = sign_transaction(
            wallet,
            built_transaction.clone(),
            SigningMethod::Fake,
        ).await;

        let signed_tx_bytes_len = consensus::serialize(&signed_transaction).len() as u64;

        if (signed_tx_bytes_len * fee_per_byte) / 1000 == total_fee {
            print(&format!("Transaction built with fee {}.", total_fee));
            return built_transaction;
        } else {
            total_fee = (signed_tx_bytes_len * fee_per_byte) / 1000;
        }
    }
}

fn build_transaction_with_fee(
    wallet: &CustodyWallet,
    own_utxos: &[Utxo],
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
    let mut input_amounts = vec![];
    let mut total_spent = 0;
    for utxo in own_utxos.iter().rev() {
        total_spent += utxo.value;
        utxos_to_spend.push(utxo);
        input_amounts.push(Amount::from_sat(utxo.value));
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
            script_pubkey: wallet.address.script_pubkey(),
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
        input_amounts: input_amounts,
    })
}

async fn sign_transaction(
    wallet: &CustodyWallet,
    mut built_transaction: BuiltTransactionOutput,
    signing: SigningMethod,
) -> Transaction
{
    let txclone = built_transaction.transaction.clone();
    let mut cache = sighash::SighashCache::new(&txclone);

    for (index, input) in built_transaction.transaction.input.iter_mut().enumerate() {
        // Clear any previous witness
        input.witness.clear();

        let amount = built_transaction.input_amounts.get(index).unwrap();

        let sighash = cache
            .p2wsh_signature_hash(index, &wallet.witness_script, amount.clone(), EcdsaSighashType::All).expect("failed to compute sighash");

        let (signature_1, signature_2) = match signing {
            SigningMethod::Fake => (vec![255; 64], vec![255; 64]),
            SigningMethod::Real => {
                let message_hash = sighash.to_byte_array().to_vec();
                // First sign with the current (custody_wallet) canister.
                let sig1 = ecdsa_api::sign_with_ecdsa(wallet.key_name.clone(), vec![], message_hash.clone()).await;
                // Then sign with the remote (fiduciary) canister.
                let res_sig2: Result<(Vec<u8>,),_>  = call(
                    wallet.fiduciary_canister,
                    "sign_for_custody",
                    (message_hash,),
                ).await;
                let sig2 = res_sig2.unwrap().0;
                (sec1_to_der(sig1), sec1_to_der(sig2))
            },
        };

        let mut signature_der_1 = signature_1;
        signature_der_1.push(SIG_HASH_TYPE.to_u32() as u8);
        let mut signature_der_2 = signature_2;
        signature_der_2.push(SIG_HASH_TYPE.to_u32() as u8);

        input.witness.push(vec![]);  // Placeholder for scriptSig
        input.witness.push(signature_der_1);
        input.witness.push(signature_der_2);
        input.witness.push(wallet.witness_script.clone().into_bytes());
    }

    built_transaction.transaction
}

// Converts a SEC1 ECDSA signature to the DER format.
fn sec1_to_der(sec1_signature: Vec<u8>) -> Vec<u8> {
    let r: Vec<u8> = if sec1_signature[0] & 0x80 != 0 {
        // r is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[..32].to_vec());
        tmp
    } else {
        // r is positive.
        sec1_signature[..32].to_vec()
    };

    let s: Vec<u8> = if sec1_signature[32] & 0x80 != 0 {
        // s is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[32..].to_vec());
        tmp
    } else {
        // s is positive.
        sec1_signature[32..].to_vec()
    };

    // Convert signature to DER.
    vec![
        vec![0x30, 4 + r.len() as u8 + s.len() as u8, 0x02, r.len() as u8],
        r,
        vec![0x02, s.len() as u8],
        s,
    ]
    .into_iter()
    .flatten()
    .collect()
}