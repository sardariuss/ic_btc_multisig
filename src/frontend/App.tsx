import icLogo                         from './assets/ic.svg';
import { createActor } from './AgentUtils';
import NumberInput from './NumberInput';
import { ActorSubclass } from '@dfinity/agent';
import mainnetLogo                    from './assets/bitcoin_mainnet.svg';
import testnetLogo                    from './assets/bitcoin_testnet.svg';
import regtestLogo                    from './assets/bitcoin_regtest.svg';
import { AuthClient }                 from '@dfinity/auth-client';

import LoadingButton from '@mui/lab/LoadingButton';
import SendIcon from '@mui/icons-material/Send';
import { Popover } from '@mui/material';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import PopupState, { bindTrigger, bindPopover } from 'material-ui-popup-state';
import TextField from '@mui/material/TextField';
import { Refresh } from '@mui/icons-material';
import IconButton from '@mui/material/IconButton';
import Box from '@mui/material/Box';

import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import Select, { SelectChangeEvent } from '@mui/material/Select';

import { frome8s } from './utils';

import { _SERVICE, network } from '../declarations/custody_wallet/custody_wallet.did';
import { canisterId, idlFactory }  from "../declarations/custody_wallet";
import React, { useEffect, useState } from 'react';
import                                './App.css';



const networkToString = (btc_network: any) => {
  if (btc_network['mainnet'] !== undefined) return "Mainnet";
  if (btc_network['regtest'] !== undefined) return "Regtest";
  if (btc_network['testnet'] !== undefined) return "Testnet";
}

function App() {

  const [authClient, setAuthClient] = useState<AuthClient | undefined>(undefined);
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>(false);
  const [walletActor,          setWalletActor         ] = useState<ActorSubclass<_SERVICE> | undefined>(undefined);
  const [bitcoinNetwork, setBitcoinNetwork] = useState<network | undefined>(undefined);
  const [walletAddress, setWalletAddress] = useState<string | undefined>(undefined);
  const [balanceSats, setBalanceSats] = useState<bigint | undefined>(undefined);
  const [destination, setDestination] = useState<string>("");
  const [amount, setAmount] = useState<bigint>(BigInt(0));
  
  // Send
  const [sending, setSending] = useState<boolean>(false);
  const [sendLoading, setSendLoading] = useState<boolean>(false);
  const [sentOutput, setSentOutput] = useState<string | undefined>(undefined);

  // Balance
  const [balanceLoading, setBalanceLoading] = useState<boolean>(false);

  const refreshAuthClient = () => {
    AuthClient.create({
      idleOptions: {
        captureScroll: true,
        idleTimeout: 900000, // 15 minutes
        disableDefaultIdleCallback: true // disable the default reload behavior
      },
    }).then(async (client) => {
      // Refresh the authentification client and status
      const is_authenticated = await client.isAuthenticated();
      setIsAuthenticated(is_authenticated);
      setAuthClient(client);
      // Set callback on idle to logout the user
      client.idleManager?.registerCallback?.(() => logout(client));
    })
    .catch((error) => {
      console.error(error);
      setAuthClient(undefined);
      setIsAuthenticated(false);
    });
  };

  const login = () => {
    authClient?.login({
      identityProvider:
        import.meta.env.DFX_NETWORK === "ic" ? 
          `https://identity.ic0.app/#authorize` : 
          `http://localhost:${import.meta.env.DFX_REPLICA_PORT}?canisterId=${import.meta.env.CANISTER_ID_INTERNET_IDENTITY}#authorize`,
      onSuccess: () => { 
        setIsAuthenticated(true);
        // navigate("/"); @todo
      },
    });
  };

  const logout = (client: AuthClient | undefined) => {
    client?.logout().then(() => {
      // Somehow if only the isAuthenticated flag is set to false, the next login will fail
      // Refreshing the auth client fixes this behavior
      refreshAuthClient();
      //navigate("/login"); @todo
    });
  }

  const refreshWalletActor = async () => {
    setWalletActor(
      await createActor({
        canisterId,
        idlFactory,
        identity: authClient?.getIdentity(), 
      })
    );
  }

  const refreshNetwork = async () => {
    let network = await walletActor?.get_network();
    console.log(network !== undefined ? networkToString(network) : "Unknown");
    setBitcoinNetwork(network);
  }

  const refreshUserWallet = async () => {
    let address = isAuthenticated ? await walletActor?.get_wallet_address() : undefined;
    let balance = address ? await walletActor?.get_balance(address) : undefined;
    setWalletAddress(address);
    setBalanceSats(balance);
  }

  const refreshBalance = () => {
    setBalanceLoading(true);
    if (walletAddress !== undefined){
      walletActor?.get_balance(walletAddress).then((balance) => {
        setBalanceSats(balance);
      }).finally(() => {
        setBalanceLoading(false);
      });
    }
  }

  const walletSend = () => {
    if (walletActor !== undefined && destination !== undefined){
      setSendLoading(true);
      walletActor?.wallet_send({destination_address: destination, amount_in_satoshi: amount}).then((tx_id) => {
        setSentOutput("Transaction sent : " + tx_id);
        refreshBalance();
      }).catch((error) => {
        setSentOutput("Transaction failed : " + error);
      }).finally(() => {
        setSendLoading(false);
      });
    }
  }

  // Refresh the auth client on page load
  useEffect(() => {
    refreshAuthClient();
  }, []);

  // Refresh the wallet actor on auth client change
  useEffect(() => {
    refreshWalletActor();
  }, [authClient]);

  // Refresh the wallet address on wallet actor change
  useEffect(() => {
    refreshNetwork();
    refreshUserWallet();
  }, [walletActor]);

  return (
    <div className="flex flex-col items-center w-full">
      {
        isAuthenticated ? 
        <div className="flex flex-col">
          <FormControl fullWidth>
            <InputLabel id="demo-simple-select-label">Bitcoin network</InputLabel>
            <Select
              labelId="demo-simple-select-label"
              id="demo-simple-select"
              value={bitcoinNetwork !== undefined ? networkToString(bitcoinNetwork) : ""}
              label="Bitcoin network"
            >
              <MenuItem value={"Mainnet"} className="flex flex-row items-center gap-x-1">
                <img src={mainnetLogo} className="w-5 h-5"></img>
                <span>Mainnet</span>
              </MenuItem>
              <MenuItem value={"Testnet"} className="flex flex-row items-center gap-x-1">
                <img src={testnetLogo} className="w-5 h-5"></img>
                <span>Testnet</span>
              </MenuItem>
              <MenuItem value={"Regtest"} className="flex flex-row items-center gap-x-1">
                <img src={regtestLogo} className="w-5 h-5"></img>
                <span>Regtest</span>
              </MenuItem>
            </Select>
          </FormControl>
          <div>
            Address : {walletAddress}
          </div>
          <div className="flex flex-row space-x-1 items-center">
            <LoadingButton
              size="small"
              onClick={(e) => refreshBalance()}
              startIcon={<Refresh />}
              loading={balanceLoading}
              variant="text"
            />
            <span>
              { balanceSats !== undefined ? frome8s(balanceSats).toFixed(8) : ""}
            </span>
            <span>
              { balanceSats !== undefined ? "btc" : "" }
            </span>
          </div>
          <div className="grid grid-cols-2 items-center">
            <span>
              Destination
            </span>
            <TextField
              id="outlined-controlled"
              label="btc address"
              value={destination}
              onChange={(event: React.ChangeEvent<HTMLInputElement>) => {
                setDestination(event.target.value);
              }}
            />
            <span>
              Amount
            </span>
            <NumberInput
              label=""
              min={0}
              initialValue={0}
              onChange={(n) => setAmount(BigInt(n))}
              unit="sats"
            />
          </div>
          <PopupState variant="popover" popupId="demo-popup-popover">
            {(popupState) => (
              <div>
                <Button variant="contained" {...bindTrigger(popupState)}>
                  Send
                </Button>
                <Popover
                  {...bindPopover(popupState)}
                  anchorOrigin={{
                    vertical: 'bottom',
                    horizontal: 'center',
                  }}
                  transformOrigin={{
                    vertical: 'top',
                    horizontal: 'center',
                  }}
                >
                  <div>
                    <Typography sx={{ p: 2 }}>Send {amount.toString()} to {destination} ?</Typography>
                    <LoadingButton
                      size="small"
                      onClick={(e) => setSending(true)}
                      endIcon={<SendIcon />}
                      loading={sendLoading}
                      variant="contained"
                    >
                      <span>Confirm</span>
                    </LoadingButton>
                  </div>
                  <div>
                    {sentOutput}
                  </div>
                </Popover>
              </div>
            )}
          </PopupState>
          <Button variant="contained" onClick={() => logout(authClient)}>
            Logout
          </Button>
        </div> :
        <div className="w-full">
          <div className="flex flex-row justify-center gap-x-5">
            <a href="https://internetcomputer.org/" target="_blank" className="ic">
              <div className="glitch">
                <img src={icLogo} className="logo" alt="" />
                <div className="glitch__layers">
                  <div className="glitch__layer"></div>
                  <div className="glitch__layer"></div>
                  <div className="glitch__layer"></div>
                </div>
              </div>
            </a>
            <a href="https://bitcoin.org/" target="_blank">
              <img src={mainnetLogo} className="logo btc" alt="Bitcoin logo" />
            </a>
          </div>
          <h1>Multi-subnet Bitcoin wallet</h1>
          <div className="card">
            <button onClick={login}>
              Login
            </button>
          </div>
          <p className="read-the-docs">
            @todo
          </p>
        </div>
      }
    </div>
  );
}

export default App;