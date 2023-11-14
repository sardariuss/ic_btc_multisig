import Types    "Types";
import EcdsaApi "EcdsaApi";

import R        "mo:base/Result";
import P        "mo:base/Principal";
import B        "mo:base/Blob";

shared actor class Fiduciary({bitcoin_network; custody_id;}: Types.FiduciaryArgs) = {

  /// Master key name, it shall differ from the one used in the custody_wallet.
  stable let _key_name = switch(bitcoin_network){
    // For local development, we use a special test key with dfx.
    case(#Regtest) "dfx_test_key";
    // On the IC we're using a test ECDSA key.
    case(#Mainnet) "key_1";
    case(#Testnet) "key_1";
  };
  /// The derivation is arbitrarily left empty.
  stable let _derivation_path = [B.fromArray([])];
  /// The custody id is the principal of the custody canister.
  stable let _custody_id      = custody_id;

  public shared func public_key(): async Blob {
    await EcdsaApi.ecdsa_public_key(_key_name, _derivation_path);
  };

  public shared({caller}) func sign_for_custody(message_hash: Blob): async R.Result<Blob, Text> {
    if (caller != _custody_id) {
      return #err("Only the custody canister '" # P.toText(_custody_id) # "'' is allowed to request a signature");
    };
    #ok(await EcdsaApi.sign_with_ecdsa(_key_name, _derivation_path, message_hash));
  };
  
};