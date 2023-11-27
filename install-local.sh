# Install the canisters locally, configured to use the regtest network

dfx start --background
dfx canister create --all

dfx build

export FIDUCIARY_ID=$(dfx canister id fiduciary)

dfx canister install fiduciary

dfx canister install custody_wallet --argument="(record {
  bitcoin_network = variant { regtest };
  fiduciary_id = principal \"${FIDUCIARY_ID}\";
})"

dfx canister install internet_identity

dfx canister install frontend