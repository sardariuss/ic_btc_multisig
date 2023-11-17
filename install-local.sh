dfx start --background
dfx canister create --all

dfx build

export CUSTODY_ID=$(dfx canister id custody_wallet)
export FIDUCIARY_ID=$(dfx canister id fiduciary)

dfx canister install fiduciary --argument="(record {
  bitcoin_network = variant { Regtest };
  custody_id = principal \"${CUSTODY_ID}\";
})"

dfx canister install custody_wallet --argument="(record {
  bitcoin_network = variant { regtest };
  fiduciary_id = principal \"${FIDUCIARY_ID}\";
})"

dfx canister install internet_identity

dfx canister install frontend