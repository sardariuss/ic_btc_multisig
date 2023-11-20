import Types     "Types";
import EcdsaApi  "EcdsaApi";

import Debug     "mo:base/Debug";
import Principal "mo:base/Principal";

shared actor class Fiduciary({custody_id;}: Types.FiduciaryArgs) = {

  type BitcoinNetwork = Types.BitcoinNetwork;

  /// The custody id is the principal of the custody canister.
  stable let _custody_id = custody_id;

  public shared func public_key(network: BitcoinNetwork, derivation_path: [Blob]): async Blob {
    await EcdsaApi.ecdsa_public_key(getKeyName(network), derivation_path)
  };

  public shared({caller}) func sign_for_custody(network: BitcoinNetwork, derivation_path: [Blob], message_hash: Blob): async Blob {
    if (caller != _custody_id) {
      Debug.trap("Only the custody canister '" # Principal.toText(_custody_id) # "'' is allowed to request a signature");
    };
    await EcdsaApi.sign_with_ecdsa(getKeyName(network), derivation_path, message_hash)
  };

  public query func get_ecdsa_key_name(network: BitcoinNetwork) : async Text {
    getKeyName(network)
  };

    /// Master key name, it shall differ from the one used in the custody_wallet.
  func getKeyName(network: BitcoinNetwork) : Text {
    switch(network){
      // For local development, we use a special test key with dfx.
      case(#regtest) "dfx_test_key";
      // On the IC we're using a test ECDSA key.
      case(#mainnet) "key_1";
      case(#testnet) "key_1";
    };
  };
  
};