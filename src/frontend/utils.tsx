import mainnetLogo                    from './assets/bitcoin_mainnet.svg';
import testnetLogo                    from './assets/bitcoin_testnet.svg';
import regtestLogo                    from './assets/bitcoin_regtest.svg';

import { HttpAgent, Identity, Actor, ActorMethod, ActorSubclass } from "@dfinity/agent";
import { IDL }                                                    from "@dfinity/candid";

export const networkToString = (btc_network: any) => {
  if (btc_network['mainnet'] !== undefined) return "Mainnet";
  if (btc_network['regtest'] !== undefined) return "Regtest";
  if (btc_network['testnet'] !== undefined) return "Testnet";
  throw new Error("Unknown network");
}

export const networkToLogo = (btc_network: any) => {
  if (btc_network['mainnet'] !== undefined) return mainnetLogo;
  if (btc_network['regtest'] !== undefined) return regtestLogo;
  if (btc_network['testnet'] !== undefined) return testnetLogo;
  throw new Error("Unknown network");
}

type CreateAgentParams = { 
  identity: Identity | undefined 
}

export const getAgent = async ({identity} : CreateAgentParams) : Promise<HttpAgent> => {
  const is_dev = import.meta.env.DFX_NETWORK !== "ic";

  let agent = new HttpAgent({ 
    host: is_dev ? `http://localhost:${import.meta.env.DFX_REPLICA_PORT}` : `https://icp0.io`,
    identity
  });

  if (is_dev) {
    // Fetch root key for certificate validation during development
    await agent.fetchRootKey();
  };

  return agent;
}

type CreateActorParams = {
  canisterId: string,
  idlFactory: IDL.InterfaceFactory,
  identity: Identity | undefined
}

export const createActor = async <T = Record<string, ActorMethod>>({canisterId, idlFactory, identity} : CreateActorParams) : Promise<ActorSubclass<T>> => {
  const agent = await getAgent({ identity });
  return Actor.createActor(idlFactory, { agent, canisterId });
}

const clampNumber = (
  val: any,
  min: number = -Infinity,
  max: number = Infinity,
  decimalScale: number = 0,
): number => {
  let v = typeof val === "number" ? val : Number(val);
  v = Math.min(max, Math.max(min, isNaN(v) ? 0 : v));
  return Number(v.toFixed(decimalScale));
};

const generateNumberRegex = (
  min: number,
  max: number,
  allowDecimal: boolean,
): RegExp => {
  const floatRegexStr = "(\\.[0-9]*)?";
  const negativeIntRegexStr = "-[0-9]*";
  const positiveIntRegexStr = "[0-9]+";
  const positiveOrNegativeIntRegexStr = "-?[0-9]*";

  let regexStr = "^";
  if (max < 0) regexStr += negativeIntRegexStr;
  else if (min > 0) regexStr += positiveIntRegexStr;
  else regexStr += positiveOrNegativeIntRegexStr;
  if (allowDecimal) regexStr += floatRegexStr;
  regexStr += "$";
  return new RegExp(regexStr);
};

const getFormControlProps = (props: any) => {
  return {
    color: props.color,
    disabled: props.disabled,
    error: props.error,
    fullWidth: props.fullWidth,
    required: props.required,
    variant: props.variant,
  };
};

const frome8s = (e8s: bigint) : number => {
  return Number(e8s) / 100000000;
}

export { clampNumber, generateNumberRegex, getFormControlProps, frome8s };
