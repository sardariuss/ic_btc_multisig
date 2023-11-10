import hashlib

from bitcoin import SelectParams
from bitcoin.core import b2x, lx, COIN, COutPoint, CMutableTxOut, CMutableTxIn, CMutableTransaction, CTxInWitness, CTxWitness
from bitcoin.core.script import CScript, CScriptWitness, OP_0, OP_2, OP_CHECKMULTISIG, SignatureHash, SIGHASH_ALL, SIGVERSION_WITNESS_V0
from bitcoin.wallet import CBitcoinSecret, CBitcoinAddress, P2WSHBitcoinAddress

# We'll be using testnet throughout this guide
SelectParams("testnet")

# Create the (in)famous correct brainwallet secret key.
# first key
h1 = hashlib.sha256(b'correct horse battery staple first').digest()
seckey1 = CBitcoinSecret.from_secret_bytes(h1)
pubkey1 = "0301abde8810babb194564c49f690a54cbe3be595838e1668950118bc2e0cc655a"

# second key
h2 = hashlib.sha256(b'correct horse battery staple second').digest()
seckey2 = CBitcoinSecret.from_secret_bytes(h2)
pubkey2 = "023134778661a1cbb8ca508734f728ca12c1a9d4e379b58ff3e491f69ceb2eb824"

# Create a witnessScript. witnessScript in SegWit is equivalent to redeemScript in P2SH transaction,
# however, while the redeemScript of a P2SH transaction is included in the ScriptSig, the 
# WitnessScript is included in the Witness field, making P2WSH inputs cheaper to spend than P2SH 
# inputs.
witness_script = CScript([OP_2, bytes.fromhex(pubkey1), bytes.fromhex(pubkey2), OP_2, OP_CHECKMULTISIG])
script_hash = hashlib.sha256(witness_script).digest()
script_pubkey = CScript([OP_0, script_hash])

# Convert the P2WSH scriptPubKey to a base58 Bitcoin address and print it.
# You'll need to send some funds to it to create a txout to spend.
address = P2WSHBitcoinAddress.from_scriptPubKey(script_pubkey)
print('Address:', str(address))
# outputs: Address: bcrt1qljlyqaexx4mmhpl66e6nqdtagjaht87pghuq6p0f98a765c9uj9s3f3ee3

# we are continuing the code from above

txid = lx("49ff22c9985c1991791b7a3bd6a2e8d1d6567ca283e0885afdc83bd92f56d1c4")
vout = 0

# Specify the amount send to your P2WSH address.
amount = int(1 * COIN)

# Calculate an amount for the upcoming new UTXO. Set a high fee (5%) to bypass bitcoind minfee
# setting on regtest.
amount_less_fee = amount * 0.99

# Create the txin structure, which includes the outpoint. The scriptSig defaults to being empty as
# is necessary for spending a P2WSH output.
txin = CMutableTxIn(COutPoint(txid, vout))

# Specify a destination address and create the txout.
destination = CBitcoinAddress("mkJ1nQaSPppu8o5srLxaRBRSQeACp49eyK").to_scriptPubKey() 
txout = CMutableTxOut(amount_less_fee, destination)

# Create the unsigned transaction.
tx = CMutableTransaction([txin], [txout])

# Calculate the signature hash for that transaction.
sighash = SignatureHash(
    script=witness_script,
    txTo=tx,
    inIdx=0,
    hashtype=SIGHASH_ALL,
    amount=amount,
    sigversion=SIGVERSION_WITNESS_V0,
)

# Now sign it. We have to append the type of signature we want to the end, in this case the usual
# SIGHASH_ALL.
sig1 = seckey1.sign(sighash) + bytes([SIGHASH_ALL])
sig2 = seckey2.sign(sighash) + bytes([SIGHASH_ALL])

# Construct a witness for this P2WSH transaction and add to tx.
witness = CScriptWitness([b"", *[sig1, sig2], witness_script])
tx.wit = CTxWitness([CTxInWitness(witness)])

# Done! Print the transaction
print(b2x(tx.serialize()))
# outputs: 01000000000101c4d1562fd93bc8fd5a88e083a27c56d6d1e8a2d63b7a1b7991195c98c922ff490000000000ffffffff01c09ee605000000001600147829e2df6fd013aa5303d4e0af578d4275629bd30400483045022100fef9da5dfaea90104f033960b00612753197bf96c69fce097699aff60261aa3402203b72ed27c929125e5aa0066e6f641b7ffaee4bb6e4929a2428e79a2e0ed4f0140148304502210094fca9f85165c024cace7e92a099be3199e427103a971fd6336ce7386bb8fe410220046b043475b72ee5f9fa2872344c689ee9c800031010db0c11fdb57b0d37ab6301475221038d19497c3922b807c91b829d6873ae5bfa2ae500f3237100265a302fdce87b052103d3a9dff5a0bb0267f19a9ee1c374901c39045fbe041c1c168d4da4ce0112595552ae00000000
