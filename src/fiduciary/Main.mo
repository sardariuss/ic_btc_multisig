import Types    "Types";
import EcdsaApi "EcdsaApi";

import D        "mo:base/Debug";
import P        "mo:base/Principal";

shared actor class Fiduciary({bitcoin_network; custody_id;}: Types.FiduciaryArgs) = {

  /// Master key name, it shall differ from the one used in the custody_wallet.
  stable let _key_name = switch(bitcoin_network){
    // For local development, we use a special test key with dfx.
    case(#Regtest) "dfx_test_key";
    // On the IC we're using a test ECDSA key.
    case(#Mainnet) "key_1";
    case(#Testnet) "key_1";
  };
  /// The custody id is the principal of the custody canister.
  stable let _custody_id      = custody_id;

  public shared func public_key(derivation_path: [Blob]): async Blob {
    await EcdsaApi.ecdsa_public_key(_key_name, derivation_path)
  };

  public shared({caller}) func sign_for_custody(derivation_path: [Blob], message_hash: Blob): async Blob {
    if (caller != _custody_id) {
      D.trap("Only the custody canister '" # P.toText(_custody_id) # "'' is allowed to request a signature");
    };
    await EcdsaApi.sign_with_ecdsa(_key_name, derivation_path, message_hash)
  };
  
};