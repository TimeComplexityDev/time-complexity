import { createContext, useContext, useState, useCallback, type ReactNode } from 'react';
import { pair as pairBridge, getStatus } from '../api/bridge';
import { getPairToken, setPairToken, clearPairToken } from '../api/storage';

interface PairingContextValue {
  token: string | null;
  isPaired: boolean;
  isPairing: boolean;
  pair: (token: string) => Promise<void>;
  unpair: () => void;
  verifyPairing: () => Promise<boolean>;
}

const PairingContext = createContext<PairingContextValue | null>(null);

export function PairingProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(getPairToken);
  const [isPairing, setIsPairing] = useState(false);
  const isPaired = token !== null;

  const pair = useCallback(async (newToken: string) => {
    setIsPairing(true);
    try {
      await pairBridge(newToken);
      setPairToken(newToken);
      setToken(newToken);
    } finally {
      setIsPairing(false);
    }
  }, []);

  const unpair = useCallback(() => {
    clearPairToken();
    setToken(null);
  }, []);

  const verifyPairing = useCallback(async (): Promise<boolean> => {
    const t = getPairToken();
    if (!t) return false;
    try {
      await getStatus();
      return true;
    } catch {
      return false;
    }
  }, []);

  return (
    <PairingContext.Provider value={{ token, isPaired, isPairing, pair, unpair, verifyPairing }}>
      {children}
    </PairingContext.Provider>
  );
}

export function usePairing(): PairingContextValue {
  const ctx = useContext(PairingContext);
  if (!ctx) throw new Error('usePairing must be used within PairingProvider');
  return ctx;
}