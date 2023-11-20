module {

  public type FiduciaryArgs = {
    custody_id: Principal;
  };

  public type ECDSAPublicKeyReply = {
    public_key : Blob;
    chain_code : Blob;
  };

  public type EcdsaCurve = {
    #secp256k1;
  };

  public type EcdsaKeyId = {
    curve : EcdsaCurve;
    name : Text;
  };

  public type SignWithECDSA = {
    message_hash : Blob;
    derivation_path : [Blob];
    key_id : EcdsaKeyId;
  };

  public type SignWithECDSAReply = {
    signature : Blob;
  };

  public type ECDSAPublicKey = {
    canister_id : ?Principal;
    derivation_path : [Blob];
    key_id : EcdsaKeyId;
  };

  public type Cycles = Nat;

  public type BitcoinNetwork = {
    #mainnet;
    #testnet;
    #regtest;
  };

};