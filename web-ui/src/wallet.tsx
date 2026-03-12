import { StrictMode, useState } from 'react';
import { createRoot } from 'react-dom/client';
import {
  ConnectWalletButton,
  DogestashProvider,
  useUnifiedWallet,
} from '@jonheaven/dogestash';

const styles = `
  .dog-wallet-shell {
    color: #fff8ec;
    font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }

  .dog-wallet-card {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.2rem;
    background: rgba(8, 12, 24, 0.78);
    padding: 1rem;
  }

  .dog-wallet-card + .dog-wallet-card {
    margin-top: 0.9rem;
  }

  .dog-wallet-kicker {
    margin: 0 0 0.5rem;
    color: rgba(248, 208, 116, 0.76);
    font-size: 0.7rem;
    font-weight: 700;
    letter-spacing: 0.22em;
    text-transform: uppercase;
  }

  .dog-wallet-heading {
    margin: 0;
    color: #fff1d1;
    font-size: 1.35rem;
    font-weight: 700;
  }

  .dog-wallet-copy {
    margin: 0.7rem 0 0;
    color: rgba(255, 255, 255, 0.62);
    font-size: 0.92rem;
    line-height: 1.55;
  }

  .dog-wallet-connect {
    width: 100%;
    border: 1px solid rgba(245, 158, 11, 0.52);
    border-radius: 0.9rem;
    background: linear-gradient(135deg, rgba(245, 158, 11, 0.96), rgba(251, 191, 36, 0.88));
    color: #1b1204;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 700;
    padding: 0.9rem 1rem;
  }

  .dog-wallet-grid {
    display: grid;
    gap: 0.75rem;
    margin-top: 0.9rem;
  }

  .dog-wallet-pane {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    background: rgba(255, 255, 255, 0.03);
    padding: 0.85rem;
  }

  .dog-wallet-row {
    align-items: center;
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .dog-wallet-label {
    color: rgba(255, 255, 255, 0.44);
    font-size: 0.76rem;
    letter-spacing: 0.12em;
    text-transform: uppercase;
  }

  .dog-wallet-value {
    color: #facc15;
    font-size: 0.85rem;
    font-weight: 700;
  }

  .dog-wallet-mono,
  .dog-wallet-result pre {
    margin: 0.55rem 0 0;
    color: rgba(255, 255, 255, 0.84);
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 0.77rem;
    line-height: 1.55;
    word-break: break-word;
  }

  .dog-wallet-balance {
    margin: 0.55rem 0 0;
    color: #fff1d1;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", monospace;
    font-size: 1.08rem;
    font-weight: 700;
  }

  .dog-wallet-title {
    margin: 0;
    color: #fff1d1;
    font-size: 0.96rem;
    font-weight: 700;
  }

  .dog-wallet-hint {
    color: rgba(255, 255, 255, 0.44);
    font-size: 0.72rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .dog-wallet-textarea,
  .dog-wallet-input {
    width: 100%;
    box-sizing: border-box;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.9rem;
    background: rgba(4, 8, 18, 0.72);
    color: #fff1d1;
    font: inherit;
    margin-top: 0.75rem;
    outline: none;
    padding: 0.82rem 0.9rem;
  }

  .dog-wallet-field-grid,
  .dog-wallet-actions {
    display: grid;
    gap: 0.75rem;
    margin-top: 0.8rem;
  }

  .dog-wallet-field-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .dog-wallet-field-label {
    color: rgba(255, 255, 255, 0.44);
    display: grid;
    font-size: 0.72rem;
    gap: 0.35rem;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .dog-wallet-button,
  .dog-wallet-actions button {
    width: 100%;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.9rem;
    background: rgba(15, 23, 42, 0.96);
    color: #ffe7b3;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 700;
    padding: 0.82rem 1rem;
  }

  .dog-wallet-actions {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .dog-wallet-button--emerald {
    background: rgba(16, 185, 129, 0.14);
    border-color: rgba(16, 185, 129, 0.24);
    color: #d1fae5;
  }

  .dog-wallet-button--sky {
    background: rgba(14, 165, 233, 0.14);
    border-color: rgba(14, 165, 233, 0.24);
    color: #dbeafe;
  }

  .dog-wallet-error {
    margin-top: 0.85rem;
    border: 1px solid rgba(248, 113, 113, 0.28);
    border-radius: 0.95rem;
    background: rgba(127, 29, 29, 0.26);
    color: #fecaca;
    font-size: 0.88rem;
    padding: 0.85rem 0.95rem;
  }

  .dog-wallet-result pre {
    min-height: 10rem;
    overflow-x: auto;
    white-space: pre-wrap;
  }

  @media (max-width: 640px) {
    .dog-wallet-field-grid,
    .dog-wallet-actions {
      grid-template-columns: minmax(0, 1fr);
    }
  }
`;

function formatBalance(balance: number, verified: boolean) {
  if (!verified) {
    return 'Balance pending';
  }

  return `${balance.toFixed(8)} DOGE`;
}

function parseInteger(value: string, fallback: number) {
  const parsed = Number.parseInt(value, 10);
  return Number.isSafeInteger(parsed) && parsed > 0 ? parsed : fallback;
}

function WalletTools() {
  const {
    connected,
    address,
    balance,
    balanceVerified,
    signMessage,
    signDMPIntent,
    walletType,
  } = useUnifiedWallet();
  const [message, setMessage] = useState('Dog monitor heartbeat: additive viewing is live.');
  const [priceKoinu, setPriceKoinu] = useState('4206900000');
  const [expiryHeight, setExpiryHeight] = useState('6000000');
  const [result, setResult] = useState('Signed output will appear here.');
  const [error, setError] = useState<string | null>(null);
  const [activeAction, setActiveAction] = useState<'message' | 'listing' | 'bid' | null>(null);

  const resetState = () => {
    setError(null);
    setResult('Signed output will appear here.');
  };

  const showResult = (nextResult: string) => {
    setError(null);
    setResult(nextResult);
  };

  const showError = (nextError: unknown, fallback: string) => {
    setError(nextError instanceof Error ? nextError.message : fallback);
  };

  const handleSignMessage = async () => {
    resetState();
    setActiveAction('message');

    try {
      showResult(await signMessage(message));
    } catch (nextError) {
      showError(nextError, 'Message signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  const handleSignListing = async () => {
    resetState();
    setActiveAction('listing');

    try {
      const signed = await signDMPIntent('listing', {
        price_koinu: parseInteger(priceKoinu, 4206900000),
        psbt_cid: 'ipfs://QmDogMonitorListing',
        expiry_height: parseInteger(expiryHeight, 6000000),
      });
      showResult(JSON.stringify(signed, null, 2));
    } catch (nextError) {
      showError(nextError, 'Listing signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  const handleSignBid = async () => {
    resetState();
    setActiveAction('bid');

    try {
      const signed = await signDMPIntent('bid', {
        listing_id: `${'b'.repeat(64)}i0`,
        price_koinu: parseInteger(priceKoinu, 4206900000),
        psbt_cid: 'ipfs://QmDogMonitorBid',
        expiry_height: parseInteger(expiryHeight, 6000000),
      });
      showResult(JSON.stringify(signed, null, 2));
    } catch (nextError) {
      showError(nextError, 'Bid signing failed');
    } finally {
      setActiveAction(null);
    }
  };

  return (
    <div className="dog-wallet-shell">
      <style>{styles}</style>

      <div className="dog-wallet-card">
        <p className="dog-wallet-kicker">Wallet Tools</p>
        <h2 className="dog-wallet-heading">Dogestash wallet lane</h2>
        <p className="dog-wallet-copy">
          Connect a supported wallet to run explorer-native signing and balance checks.
        </p>

        <div style={{ marginTop: '0.9rem' }}>
          <ConnectWalletButton
            className="dog-wallet-connect"
            connectLabel="Connect Wallet"
            disconnectLabel="Disconnect"
          />
        </div>

        <div className="dog-wallet-grid">
          <div className="dog-wallet-pane">
            <div className="dog-wallet-row">
              <span className="dog-wallet-label">Wallet</span>
              <span className="dog-wallet-value">{walletType ?? 'not connected'}</span>
            </div>
            <p className="dog-wallet-mono">
              {address ?? 'Connect a wallet to enable message signing and DMP test intents.'}
            </p>
          </div>

          <div className="dog-wallet-pane">
            <div className="dog-wallet-row">
              <span className="dog-wallet-label">Balance check</span>
              <span className="dog-wallet-value">
                {balanceVerified ? 'verified' : 'awaiting provider'}
              </span>
            </div>
            <p className="dog-wallet-balance">{formatBalance(balance, balanceVerified)}</p>
          </div>
        </div>
      </div>

      <div className="dog-wallet-card">
        <div className="dog-wallet-row">
          <h3 className="dog-wallet-title">Sign message</h3>
          <span className="dog-wallet-hint">CLI parity</span>
        </div>
        <textarea
          className="dog-wallet-textarea"
          rows={4}
          value={message}
          onChange={(event) => setMessage(event.target.value)}
        />
        <button
          className="dog-wallet-button"
          disabled={!connected || activeAction !== null}
          onClick={handleSignMessage}
          type="button"
        >
          {activeAction === 'message' ? 'Signing...' : 'Sign Message'}
        </button>
      </div>

      <div className="dog-wallet-card">
        <div className="dog-wallet-row">
          <h3 className="dog-wallet-title">DMP intents</h3>
          <span className="dog-wallet-hint">listing + bid</span>
        </div>
        <div className="dog-wallet-field-grid">
          <label className="dog-wallet-field-label">
            price_koinu
            <input
              className="dog-wallet-input"
              value={priceKoinu}
              onChange={(event) => setPriceKoinu(event.target.value)}
            />
          </label>
          <label className="dog-wallet-field-label">
            expiry_height
            <input
              className="dog-wallet-input"
              value={expiryHeight}
              onChange={(event) => setExpiryHeight(event.target.value)}
            />
          </label>
        </div>
        <div className="dog-wallet-actions">
          <button
            className="dog-wallet-button dog-wallet-button--emerald"
            disabled={!connected || activeAction !== null}
            onClick={handleSignListing}
            type="button"
          >
            {activeAction === 'listing' ? 'Signing listing...' : 'Sign Listing'}
          </button>
          <button
            className="dog-wallet-button dog-wallet-button--sky"
            disabled={!connected || activeAction !== null}
            onClick={handleSignBid}
            type="button"
          >
            {activeAction === 'bid' ? 'Signing bid...' : 'Sign Bid'}
          </button>
        </div>
      </div>

      {error ? <div className="dog-wallet-error">{error}</div> : null}

      <div className="dog-wallet-card dog-wallet-result">
        <div className="dog-wallet-row">
          <h3 className="dog-wallet-title">Result</h3>
          <span className="dog-wallet-hint">{connected ? 'live wallet output' : 'connect to test'}</span>
        </div>
        <pre>{result}</pre>
      </div>
    </div>
  );
}

function App() {
  return (
    <StrictMode>
      <DogestashProvider>
        <WalletTools />
      </DogestashProvider>
    </StrictMode>
  );
}

const mountNode = document.getElementById('wallet-tools-root');

if (mountNode) {
  createRoot(mountNode).render(<App />);
}
