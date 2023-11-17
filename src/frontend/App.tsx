import                                             './App.css';
import icLogo                                      from './assets/ic.svg';
import mainnetLogo                                 from './assets/bitcoin_mainnet.svg';
import testnetLogo                                 from './assets/bitcoin_testnet.svg';
import regtestLogo                                 from './assets/bitcoin_regtest.svg';

import { frome8s, networkToString, networkToLogo } from './utils';
import { network }                                 from '../declarations/custody_wallet/custody_wallet.did';

import NumberInput                                 from './components/NumberInput';
import Title                                       from './components/Title';
import { Context, useContext }                     from './components/Context';
import TextSnackbar                                from './components/TextSnackbar';

import LoadingButton                               from '@mui/lab/LoadingButton';
import TabContext                                  from '@mui/lab/TabContext';
import TabList                                     from '@mui/lab/TabList';
import TabPanel                                    from '@mui/lab/TabPanel';

import SendIcon                                    from '@mui/icons-material/Send';
import { Refresh }                                 from '@mui/icons-material';
import CopyIcon                                    from '@mui/icons-material/FileCopy';

import PopupState, { bindTrigger, bindPopover }    from 'material-ui-popup-state';
import { Popover }                                 from '@mui/material';
import Button                                      from '@mui/material/Button';
import TextField                                   from '@mui/material/TextField';
import IconButton                                  from '@mui/material/IconButton';
import Box                                         from '@mui/material/Box';
import CircularProgress                            from '@mui/material/CircularProgress';
import Tab                                         from '@mui/material/Tab';

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
    login,
    logout,
  } = useContext();

  // Wallet
  const [bitcoinNetwork, setBitcoinNetwork] = useState<network | undefined>(undefined);
  const [walletAddress,  setWalletAddress ] = useState<string | undefined> (undefined);
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

  const refreshUserWallet = async () => {
    let address = isAuthenticated ? await walletActor?.get_wallet_address() : undefined;
    let balance = address ? await walletActor?.get_balance(address) : undefined;
    setWalletAddress(address);
    setBalanceSats(balance);
  }

  const refreshBalance = () => {
    setBalanceSats(undefined);
    if (walletAddress !== undefined){
      walletActor?.get_balance(walletAddress).then((balance) => {
        setBalanceSats(balance);
      }).finally(() => {
      });
    }
  }

  const walletSend = () => {
    if (walletActor !== undefined && destination !== undefined){
      setSendLoading(true);
      walletActor?.wallet_send({destination_address: destination, amount_in_satoshi: amount}).then((tx_id) => {
        setSentSuccess(true);
        setSentOutput("Transaction id: " + tx_id);
        refreshBalance();
      }).catch((error) => {
        setSentSuccess(false);
        setSentOutput("Error: " + error);
      }).finally(() => {
        setSendLoading(false);
      });
    }
  }

  // Refresh the wallet address on wallet actor change
  useEffect(() => {
    refreshNetwork();
    refreshUserWallet();
  }, [walletActor]);

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
        {
          isAuthenticated ? 
          <div className="flex flex-col items-center w-full grow">
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
            <div className="flex flex-col w-1/3 items-center grow">
              <FormControl fullWidth>
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
              <div className="py-4">
                {
                  balanceSats === undefined ?
                  <CircularProgress size={24}/> :
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
                            <Button size="large" variant="contained" {...bindTrigger(popupState)} disabled={destination.length === 0 || amount < 1}>
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
                                <div className="flex flex-col items-center gap-y-5 break-all">
                                  <span className="text-xl">Send {frome8s(amount).toFixed(8)} btc to {destination} ?</span>
                                  <LoadingButton
                                    size="large"
                                    onClick={(e) => { walletSend(); }}
                                    endIcon={<SendIcon />}
                                    loading={sendLoading}
                                    variant="contained"
                                  >
                                    <span>Confirm</span>
                                  </LoadingButton>
                                </div>
                                <TextSnackbar success={sentSuccess} message={sentOutput} setMessage={setSentOutput}/>
                              </Box>
                            </Popover>
                          </div>
                        )}
                      </PopupState>
                    </div>
                  </TabPanel>
                  <TabPanel value="Receive">
                    {
                      walletAddress === undefined ? <></> :
                      <div className="flex flex-col items-center gap-y-5">
                        <span className="self-start">BTC {networkToString(bitcoinNetwork)} address:</span>
                        <div className="flex flex-row gap-x-1 items-center">
                          <span className="text-xl break-all">{walletAddress}</span>
                          <IconButton onClick={(e) => navigator.clipboard.writeText(walletAddress)}>
                            <CopyIcon />
                          </IconButton>
                        </div>
                        <QRCodeSVG value={"bitcoin:" + walletAddress} height={300} width={300} imageSettings={{src: networkToLogo(bitcoinNetwork), height:25, width:25, excavate: true}}/>
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
    </Context.Provider>
  );
}

export default App;