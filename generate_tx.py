import hashlib

from bitcoin import SelectParams
from bitcoin.core import b2x, lx, COIN, COutPoint, CMutableTxOut, CMutableTxIn, CMutableTransaction, CTxInWitness, CTxWitness
from bitcoin.core.script import CScript, CScriptWitness, OP_0, OP_2, OP_CHECKMULTISIG, SignatureHash, SIGHASH_ALL, SIGVERSION_WITNESS_V0
from bitcoin.wallet import CBitcoinSecret, CBitcoinAddress, P2WSHBitcoinAddress

# We'll be using testnet throughout this guide
SelectParams("testnet")

# Create the (in)famous correct brainwallet secret key.
# first key
seckey1 = CBitcoinSecret.from_secret_bytes(bytes.fromhex("03d9cd11d73f84fcd33308143eaffa3e9b3b353f85ec5e608e5ce81d0eecb5aa"))

# second key
seckey2 = CBitcoinSecret.from_secret_bytes(bytes.fromhex("c0395f033eaace8cb17cc3249c56f706a5490daeeebc0476ae18a73c21d8c63f"))

# Create a witnessScript. witnessScript in SegWit is equivalent to redeemScript in P2SH transaction,
# however, while the redeemScript of a P2SH transaction is included in the ScriptSig, the 
# WitnessScript is included in the Witness field, making P2WSH inputs cheaper to spend than P2SH 
# inputs.
witness_script = CScript([OP_2, seckey1.pub, seckey2.pub, OP_2, OP_CHECKMULTISIG])
script_hash = hashlib.sha256(witness_script).digest()
script_pubkey = CScript([OP_0, script_hash])

# Convert the P2WSH scriptPubKey to a base58 Bitcoin address and print it.
# You'll need to send some funds to it to create a txout to spend.
address = P2WSHBitcoinAddress.from_scriptPubKey(script_pubkey)
print('Address:', str(address))
# outputs: Address: tb1qjzrkp6ms3ghxdx2mq9mkr3gaap880swq4v4w7cnaglnk75p8epzq7d5yag

# we are continuing the code from above

# https://testnet.bitcoinexplorer.org/tx/e46ded244fe701ad4a7ccaf5d793136e13abf6fae4af5118820d62c8361bcbf4
txid = lx("e46ded244fe701ad4a7ccaf5d793136e13abf6fae4af5118820d62c8361bcbf4")
vout = 0

# Specify the amount send to your P2WSH address.
amount = int(200000)

# Create the txin structure, which includes the outpoint. The scriptSig defaults to being empty as
# is necessary for spending a P2WSH output.
txin = CMutableTxIn(COutPoint(txid, vout))

# Specify a destination address and create the txout.
destination_0 = CBitcoinAddress("mkJ1nQaSPppu8o5srLxaRBRSQeACp49eyK").to_scriptPubKey()
txout_0 = CMutableTxOut(5000, destination_0)
txout_1 = CMutableTxOut(95000, address.to_scriptPubKey()) # Fee of 100000

# Create the unsigned transaction.
tx = CMutableTransaction([txin], [txout_0, txout_1])

# Calculate the signature hash for that transaction.
sighash = SignatureHash(
    script=witness_script,
    txTo=tx,
    inIdx=0,
    hashtype=SIGHASH_ALL,
    amount=amount,
    sigversion=SIGVERSION_WITNESS_V0,
)

print('sighash:', sighash.hex())

# Now sign it. We have to append the type of signature we want to the end, in this case the usual
# SIGHASH_ALL.
sig1 = seckey1.sign(sighash) + bytes([SIGHASH_ALL])
sig2 = seckey2.sign(sighash) + bytes([SIGHASH_ALL])

# Construct a witness for this P2WSH transaction and add to tx.
witness = CScriptWitness([b"", *[sig1, sig2], witness_script])
tx.wit = CTxWitness([CTxInWitness(witness)])

# Done! Print the transaction
print(b2x(tx.serialize()))
# outputs: 01000000000101f4cb1b36c8620d821851afe4faf6ab136e1393d7f5ca7c4aad01e74f24ed6de40000000000ffffffff02a00f0000000000001976a9143466223e25276af3fae4d3ba3706f08228147dc388acb036000000000000220020908760eb708a2e66995b017761c51de84e77c1c0ab2aef627d47e76f5027c8440400473044022077207b4c321da5d5f0ebbe85a271c964ff9057d72da02c2c026ef860c5a643750220764ebab188b2853e0190c9e41ff59a41e3ead7230563ac56f214ce6eafdd5b2a01483045022100e8ad4476b30d77068b612518188b8a2a7b317ab7f5d56ee414520d9644e3110e022002e1d5c22bb94b380f5cb31f4432b70b4fa7bd65d267c6ae97cc9c798f9fd39c014752210301abde8810babb194564c49f690a54cbe3be595838e1668950118bc2e0cc655a21023134778661a1cbb8ca508734f728ca12c1a9d4e379b58ff3e491f69ceb2eb82452ae00000000
