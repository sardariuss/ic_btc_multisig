import                                             './App.css';
import icLogo                                      from './assets/ic.svg';
import mainnetLogo                                 from './assets/bitcoin_mainnet.svg';
import testnetLogo                                 from './assets/bitcoin_testnet.svg';
import regtestLogo                                 from './assets/bitcoin_regtest.svg';

import { frome8s, networkToString, networkToLogo } from './utils';
import { canisterId as walletId }                  from "../declarations/custody_wallet";
import { canisterId as fiduciaryId }               from "../declarations/fiduciary";
import { network }                                 from '../declarations/custody_wallet/custody_wallet.did';

import NumberInput                                 from './components/NumberInput';
import Title                                       from './components/Title';
import { Context, useContext }                     from './components/Context';

import LoadingButton                               from '@mui/lab/LoadingButton';
import TabContext                                  from '@mui/lab/TabContext';
import TabList                                     from '@mui/lab/TabList';
import TabPanel                                    from '@mui/lab/TabPanel';

import SendIcon                                    from '@mui/icons-material/Send';
import { Refresh, Search }                         from '@mui/icons-material';
import CopyIcon                                    from '@mui/icons-material/FileCopy';

import PopupState, { bindTrigger, bindPopover }    from 'material-ui-popup-state';
import { Popover }                                 from '@mui/material';
import Button                                      from '@mui/material/Button';
import TextField                                   from '@mui/material/TextField';
import IconButton                                  from '@mui/material/IconButton';
import Box                                         from '@mui/material/Box';
import CircularProgress                            from '@mui/material/CircularProgress';
import Tab                                         from '@mui/material/Tab';
import Alert                                       from '@mui/material/Alert';
import Table                                       from '@mui/material/Table';
import TableBody                                   from '@mui/material/TableBody';
import TableCell                                   from '@mui/material/TableCell';
import TableContainer                              from '@mui/material/TableContainer';
import TableHead                                   from '@mui/material/TableHead';
import TableRow                                    from '@mui/material/TableRow';
import Paper                                       from '@mui/material/Paper';

import { QRCodeSVG }                               from 'qrcode.react';

import InputLabel                                  from '@mui/material/InputLabel';
import MenuItem                                    from '@mui/material/MenuItem';
import FormControl                                 from '@mui/material/FormControl';
import Select                                      from '@mui/material/Select';

import React, { useEffect, useState }              from 'react';

function App() {

  const {
    authClient,
    isAuthenticated,
    walletActor,
    fiduciaryActor,
    login,
    logout,
  } = useContext();

  // Wallet general info
  const [bitcoinNetwork, setBitcoinNetwork] = useState<network | undefined>(undefined);
  const [walletKey,      setWalletKey     ] = useState<string | undefined> (undefined);
  const [fiduciaryKey,   setFiduciaryKey  ] = useState<string | undefined> (undefined);

  // User info
  const [userAddress,    setUserAddress   ] = useState<string | undefined> (undefined);
  const [balanceSats,    setBalanceSats   ] = useState<bigint | undefined> (undefined);

  // Toggle between tabs
  const [activeTab,      setActiveTab     ] = useState<string>             ("Send"   );
  
  // Send
  const [destination,    setDestination   ] = useState<string>             (""       );
  const [amount,         setAmount        ] = useState<bigint>             (BigInt(0));
  const [sendLoading,    setSendLoading   ] = useState<boolean>            (false    );
  const [sentSuccess,    setSentSuccess   ] = useState<boolean>            (false    );
  const [sentOutput,     setSentOutput    ] = useState<string>             (""       );

  const refreshNetwork = async () => {
    let network = await walletActor?.get_network();
    setBitcoinNetwork(network);
  }

  const refreshWalletKey = async () => {
    var key = undefined;
    if (bitcoinNetwork !== undefined){
      key = await walletActor?.get_ecdsa_key_name(bitcoinNetwork);
    }
    setWalletKey(key);
  }

  const refreshFiduciaryKey = async () => {
    var key = undefined;
    if (bitcoinNetwork !== undefined){
      key = await fiduciaryActor?.get_ecdsa_key_name(bitcoinNetwork);
    }
    setFiduciaryKey(key);
  }

  const refreshUserAddress = async () => {
    let address = isAuthenticated ? await walletActor?.get_wallet_address() : undefined;
    setUserAddress(address);
  }

  const refreshBalance = () => {
    setBalanceSats(undefined);
    if (userAddress !== undefined){
      walletActor?.get_balance(userAddress).then((balance) => {
        setBalanceSats(balance);
      });
    }
  }

  const walletSend = () => {
    if (walletActor !== undefined && destination !== undefined){
      setSendLoading(true);
      walletActor?.wallet_send({destination_address: destination, amount_in_satoshi: amount}).then((tx_id) => {
        setSentSuccess(true);
        setSentOutput(tx_id);
        refreshBalance();
      }).catch((error) => {
        setSentSuccess(false);
        setSentOutput(error.toString());
      }).finally(() => {
        setSendLoading(false);
      });
    }
  }

  // Refresh the wallet address on wallet actor change
  useEffect(() => {
    refreshNetwork();  
    refreshUserAddress();
  }, [walletActor]);

  // Refresh the keys on bitcoin network change
  useEffect(() => {
    refreshWalletKey();
    refreshFiduciaryKey();
  }, [bitcoinNetwork]);

  // Refresh the balance on user address change
  useEffect(() => {
    refreshBalance();
  }, [userAddress]);

  if (!authClient) return null;

  return (
    <Context.Provider value={{
      authClient,
      isAuthenticated,
      walletActor,
      login,
      logout,
    }}>
      <div className="flex flex-col items-center w-full min-h-screen grow">
        <div className="flex flex-col items-center w-full grow">
          {
            isAuthenticated ? 
            <div className="flex flex-col items-center w-full grow my-2">
              <div className="py-6">
                <Title/>
                <div className="flex flex-row justify-center gap-x-5">
                  <a href="https://internetcomputer.org/" target="_blank" className="h-12 w-24">
                    <img src={icLogo} className="logo ic" alt="" />
                  </a>
                  <a href="https://bitcoin.org/" target="_blank" className="h-12 w-12">
                    <img src={mainnetLogo} className="logo btc" alt="Bitcoin logo" />
                  </a>
                </div>
              </div>
              <div className="flex flex-col w-1/3 items-center grow space-y-2">
                <FormControl fullWidth disabled={true}>
                  <InputLabel id="demo-simple-select-label">Bitcoin network</InputLabel>
                  <Select
                    labelId="demo-simple-select-label"
                    id="demo-simple-select"
                    value={bitcoinNetwork !== undefined ? networkToString(bitcoinNetwork) : ""}
                    label="Bitcoin network"
                  >
                    <MenuItem value={"Mainnet"}>
                      <div className="flex flex-row items-center gap-x-1">
                        <img src={mainnetLogo} className="w-5 h-5"></img>
                        <span>Mainnet</span>
                      </div>
                    </MenuItem>
                    <MenuItem value={"Testnet"}>
                      <div className="flex flex-row items-center gap-x-1">
                        <img src={testnetLogo} className="w-5 h-5"></img>
                        <span>Testnet</span>
                      </div>
                    </MenuItem>
                    <MenuItem value={"Regtest"}>
                      <div className="flex flex-row items-center gap-x-1">
                        <img src={regtestLogo} className="w-5 h-5"></img>
                        <span>Regtest</span>
                      </div>
                    </MenuItem>
                  </Select>
                </FormControl>
                <TableContainer component={Paper}>
                  <Table aria-label="simple table" size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell align="left">Canister name</TableCell>
                        <TableCell align="center">Canister ID</TableCell>
                        <TableCell align="center">ECDSA Key</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      <TableRow
                        key={0}
                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                      >
                        <TableCell align="left">
                          <span className="italic">Wallet</span>
                        </TableCell>
                        <TableCell align="center">
                          {
                            walletId === undefined ? <CircularProgress size={16}/> : 
                            <a className="text-blue-500 underline" href={"https://dashboard.internetcomputer.org/canister/" + walletId} target="_blank">{walletId}</a>
                          }
                        </TableCell>
                        <TableCell align="center">
                          {
                            walletKey === undefined ? <CircularProgress size={16}/> : 
                            <span className="font-bold"> {walletKey} </span>
                          }
                        </TableCell>
                      </TableRow>
                      <TableRow
                        key={1}
                        sx={{ '&:last-child td, &:last-child th': { border: 0 } }}
                      >
                        <TableCell align="left">
                          <span className="italic">Fiduciary</span>
                          
                        </TableCell>
                        <TableCell align="center">
                          {
                            fiduciaryId === undefined ? <CircularProgress size={16}/> : 
                            <a className="text-blue-500 underline" href={"https://dashboard.internetcomputer.org/canister/" + fiduciaryId} target="_blank">{fiduciaryId}</a>
                          }
                        </TableCell>
                        <TableCell align="center">
                          {
                            fiduciaryKey === undefined ? <CircularProgress size={16}/> : 
                            <span className="font-bold"> {fiduciaryKey} </span>
                          }
                        </TableCell>
                      </TableRow>
                    </TableBody>
                  </Table>
                </TableContainer>
                <div className="py-4">
                  {
                    balanceSats === undefined ?
                    <div className="flex flex-col items-center space-y-1">
                      <span className="text-lg">Loading balance...</span>
                      <CircularProgress size={32}/>
                    </div> :
                    <div className="flex flex-row space-x-1 items-center justify-center py-4">
                      <IconButton onClick={(e) => refreshBalance()}>
                        <Refresh />
                      </IconButton>
                      <span className="text-3xl">
                        { balanceSats !== undefined ? frome8s(balanceSats).toFixed(8) : ""}
                      </span>
                      <span className="text-xl self-end">
                        { balanceSats !== undefined ? "btc" : "" }
                      </span>
                    </div>
                  }
                </div>
                <div className="flex flex-col grow">
                  <TabContext value={activeTab}>
                    <div className="flex flex-row space-x-1 items-center justify-center w-full">
                      <TabList onChange={(event, tab) => setActiveTab(tab)} aria-label="tabs">
                        <Tab label="Send"    value="Send"    sx={{width:300, fontSize:16, fontWeight: "bold"}}/>
                        <Tab label="Receive" value="Receive" sx={{width:300, fontSize:16, fontWeight: "bold"}}/>
                      </TabList>
                    </div>
                    <TabPanel value="Send">
                      <div className="flex flex-col items-center gap-y-5">
                        <div className="grid grid-cols-2 items-center space-y-2">
                          <span className="text-lg">
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
                          <span className="text-lg">
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
                              <Button size="large" variant="contained" {...bindTrigger(popupState)} disabled={destination.length === 0 || amount < 1} onFocus={(e) => { setSentOutput("") }}>
                                Send
                              </Button>
                              <Popover
                                {...bindPopover(popupState)}
                                anchorOrigin={{
                                  vertical: 'center',
                                  horizontal: 'center',
                                }}
                                transformOrigin={{
                                  vertical: 'center',
                                  horizontal: 'center',
                                }}
                              >
                                <Box sx={{ px: 5, py: 2, m: 5 }}>
                                  <div className="flex flex-col items-center w-full gap-y-5">
                                    <span className="text-xl break-all">Send {frome8s(amount).toFixed(8)} btc to {destination} ?</span>
                                    <LoadingButton
                                      size="large"
                                      onClick={(e) => { walletSend(); }}
                                      endIcon={<SendIcon />}
                                      loading={sendLoading}
                                      variant="contained"
                                    >
                                      <span>Confirm</span>
                                    </LoadingButton>
                                    {
                                      sentOutput === "" ? <></> :
                                      <Alert variant="outlined" severity={sentSuccess ? "success" : "error"} sx={{width: 300}}
                                        action={
                                          sentSuccess ?
                                          <IconButton onClick={(e) => { window.open("https://testnet.bitcoinexplorer.org/tx/" + sentOutput, '_blank'); }}>
                                            <Search />
                                          </IconButton> :
                                          <></>
                                        }>
                                        {
                                          sentSuccess ?
                                          <div className="flex flex-col gap-y-1">
                                            <span>{"Transaction id: "}</span>
                                            <span className="break-all">{sentOutput}</span>
                                          </div> :
                                          <span className="word-break">{sentOutput}</span>
                                        }
                                      </Alert>
                                    }
                                  </div>
                                </Box>
                              </Popover>
                            </div>
                          )}
                        </PopupState>
                      </div>
                    </TabPanel>
                    <TabPanel value="Receive">
                      {
                        userAddress === undefined ? <></> :
                        <div className="flex flex-col items-center gap-y-5">
                          <span className="self-start">BTC {networkToString(bitcoinNetwork)} address:</span>
                          <div className="flex flex-row gap-x-1 items-center">
                            <span className="text-xl break-all">{userAddress}</span>
                            <IconButton onClick={(e) => navigator.clipboard.writeText(userAddress)}>
                              <CopyIcon />
                            </IconButton>
                          </div>
                          <QRCodeSVG value={"bitcoin:" + userAddress} height={300} width={300} imageSettings={{src: networkToLogo(bitcoinNetwork), height:25, width:25, excavate: true}}/>
                        </div>
                      }
                    </TabPanel>
                  </TabContext>
                </div>
                <div className="mb-10">
                  <Button variant="outlined" size="large" onClick={() => logout(authClient)}>
                    Logout
                  </Button>
                </div>
              </div>
            </div> :
            <div className="w-full flex flex-col items-center grow justify-center gap-y-10">
              <div className="flex flex-row justify-center gap-x-5">
                <a href="https://internetcomputer.org/" target="_blank" className="h-24 w-48">
                  <img src={icLogo} className="logo ic" alt="" />
                </a>
                <a href="https://bitcoin.org/" target="_blank">
                  <img src={mainnetLogo} className="logo btc h-24 w-24" alt="Bitcoin logo"/>
                </a>
              </div>
              <Title/>
              <Button variant="outlined" size="large" onClick={login}>
                Login
              </Button>
            </div>
          }
        </div>
        <footer className="flex flex-row self-end">
          <a className="fill-white h-8 w-8 my-2 mx-5" href="https://github.com/sardariuss/ic_btc_multisig" target="_blank">
            { /* Github logo */ }
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/></svg>
          </a>
        </footer>
      </div>
    </Context.Provider>
  );
}

export default App;