import { HttpAgent, Identity, Actor, ActorMethod, ActorSubclass } from "@dfinity/agent";
import { IDL }                                                    from "@dfinity/candid";

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