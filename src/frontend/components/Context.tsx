import { createActor }                from '../utils';
import { _SERVICE }                   from '../../declarations/custody_wallet/custody_wallet.did';
import { canisterId, idlFactory }     from "../../declarations/custody_wallet";

import { ActorSubclass }              from '@dfinity/agent';
import { AuthClient }                 from '@dfinity/auth-client';
import React, { useEffect, useState } from 'react';

export const Context = React.createContext<{
    authClient?: AuthClient,
    isAuthenticated: boolean,
    walletActor?: ActorSubclass<_SERVICE>,
    login: () => void,
    logout: (client: AuthClient | undefined) => void,
  }>({
    isAuthenticated: false,
    login: () => {},
    logout: () => {},
  });

export const useContext = () => {

  const [authClient,      setAuthClient     ] = useState<AuthClient | undefined>             (undefined);
  const [isAuthenticated, setIsAuthenticated] = useState<boolean>                            (false    );
  const [walletActor,     setWalletActor    ] = useState<ActorSubclass<_SERVICE> | undefined>(undefined);

  const refreshAuthClient = () => {
    AuthClient.create({
      idleOptions: {
        captureScroll: true,
        idleTimeout: 1800000, // 30 minutes
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
      },
    });
  };

  const logout = (client: AuthClient | undefined) => {
    client?.logout().then(() => {
      // Somehow if only the isAuthenticated flag is set to false, the next login will fail
      // Refreshing the auth client fixes this behavior
      refreshAuthClient();
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

  // Refresh the auth client on page load
  useEffect(() => {
    refreshAuthClient();
  }, []);

  // Refresh the wallet actor on auth client change
  useEffect(() => {
    refreshWalletActor();
  }, [isAuthenticated]);

  return {
    authClient,
    isAuthenticated,
    walletActor,
    login,
    logout,
  };
}
