# Watchout, this script install the canisters on the same application subnet
# See readme.md for more information

dfx canister create --all --ic

dfx build --ic

export CUSTODY_ID=$(dfx canister id custody_wallet --ic)
export FIDUCIARY_ID=$(dfx canister id fiduciary --ic)

dfx canister install fiduciary --ic --argument="(record {
  custody_id = principal \"${CUSTODY_ID}\";
})"

dfx canister install custody_wallet --ic --argument="(record {
  bitcoin_network = variant { testnet };
  fiduciary_id = principal \"${FIDUCIARY_ID}\";
})"

dfx canister install frontend --ic