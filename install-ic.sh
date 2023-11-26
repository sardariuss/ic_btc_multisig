# Watchout, this script install the canisters on the same application subnet
# See readme.md for more information

#dfx canister create --all --ic

#dfx build --ic
#
#dfx canister install fiduciary --ic --mode=reinstall

export FIDUCIARY_ID=$(dfx canister id fiduciary --ic)

dfx canister install custody_wallet --ic --argument="(record {
  bitcoin_network = variant { testnet };
  fiduciary_id = principal \"${FIDUCIARY_ID}\";
})" --mode=reinstall

#dfx canister install frontend --ic --mode=upgrade