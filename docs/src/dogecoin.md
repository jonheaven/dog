# Dogecoin Port

This repository is a fork of [ordinals/ord](https://github.com/ordinals/ord) v0.25.0, ported
to support Dogecoin (and Dogecoin testnet/regtest).  It incorporates the protocol-level work
from the two earlier community forks and extends them significantly:

| Codebase | Based on | Status |
|---|---|---|
| [apezord/ord-dogecoin](https://github.com/apezord/ord-dogecoin) | ord ~v0.5 | Abandoned March 2023 |
| [verydogelabs/wonky-ord-dogecoin](https://github.com/verydogelabs/wonky-ord-dogecoin) | ord ~v0.5 | Stale Oct 2024 |
| **this repo** | **ord v0.25.0** | **Active** |

---

## What was changed and why

### 1. Chain configuration (`src/chain.rs`)

Three new `Chain` variants were added: `Dogecoin`, `DogecoinTestnet`, `DogecoinRegtest`.

Key values per variant:

| Field | Dogecoin | DogecoinTestnet | DogecoinRegtest |
|---|---|---|---|
| RPC port | 22555 | 44555 | 18444 |
| First inscription height | 4,600,000 | 4,250,000 | 0 |
| Data dir subdirectory | (root) | testnet3 | regtest |
| `Network` mapping | `Network::Bitcoin` | `Network::Bitcoin` | `Network::Bitcoin` |

Dogecoin uses the same wire format as Bitcoin, so all three variants map to
`Network::Bitcoin` for the bitcoin crate.  Genesis block bytes are hardcoded as hex strings.

#### DIY address encoding

The `bitcoin` crate (v0.32.x) has no built-in Dogecoin network support.  Rather than
forking or downgrading the crate, Dogecoin addresses are encoded with a small helper:

```rust
fn dogecoin_base58check(version: u8, payload: &[u8]) -> String {
  let mut data = vec![version];
  data.extend_from_slice(payload);
  bitcoin::base58::encode_check(&data)   // appends 4-byte SHA256d checksum
}
```

Version bytes:
- `0x1e` (30) → P2PKH → addresses starting with `D`
- `0x16` (22) → P2SH  → addresses starting with `A`

---

### 2. Node / settings (`src/settings.rs`)

- **Default data directory** — `.dogecoin` (Linux/macOS) or `Dogecoin` (Windows) instead of `.bitcoin`
- **Cookie file** — `~/.dogecoin/.cookie`
- **Chain detection** — Dogecoin Core's `getblockchaininfo` returns `chain: "main"` (not
  `"bitcoin"`).  The settings layer checks `ord_chain.is_dogecoin()` first and maps
  `"main"` → `Chain::Dogecoin`, `"test"` → `Chain::DogecoinTestnet`, etc.

---

### 3. Epoch / subsidy math (`crates/ordinals/src/epoch.rs`, `lib.rs`, `sat.rs`)

Dogecoin's early block reward history is uniquely irregular.  Blocks 0–144,999 (the
"wonky era") each received a random reward between 0 and 1,000,000 DOGE.  Beyond that,
a conventional halving schedule applies until block 600,000 where the reward floors
permanently at 10,000 DOGE.

#### Constants

```rust
pub const SUBSIDY_HALVING_INTERVAL: u32 = 1;  // each block is its own epoch
pub const DIFFCHANGE_INTERVAL: u32 = 1;        // Dogecoin adjusts difficulty every block
```

Setting `SUBSIDY_HALVING_INTERVAL = 1` lets the existing epoch machinery work without
modification: `Epoch(n)` corresponds to block `n`.

#### Data files (loaded at compile time)

Two JSON files in the repository root are embedded via `include_str!`:

- **`subsidies.json`** — per-block reward in shiboshis for every wonky-era block.
  Format: `{"epochs": {"0": 8800000000, "1": 6841600000000, ...}}`

- **`starting_sats.json`** — cumulative shiboshi totals at each block boundary.
  Format: `[0, 8800000000, 6850400000000, ...]`

These are sourced from [verydogelabs/wonky-ord-dogecoin](https://github.com/verydogelabs/wonky-ord-dogecoin)
and are the community-accepted ground truth for Dogecoin ordinal numbering.

#### Post-wonky halving schedule

| Block range | Reward per block |
|---|---|
| 145,000 – 199,999 | 500,000 DOGE |
| 200,000 – 299,999 | 250,000 DOGE |
| 300,000 – 399,999 | 125,000 DOGE |
| 400,000 – 499,999 | 62,500 DOGE |
| 500,000 – 599,999 | 31,250 DOGE |
| 600,000+ | **10,000 DOGE (permanent floor)** |

#### Supply ceiling

Dogecoin has no hard supply cap.  `Sat::SUPPLY` is set to
`180_000_000_000 * COIN_VALUE` (180 billion DOGE in shiboshis) as a practical
ceiling — the maximum that fits in a `u64` while leaving headroom above the
current ~140B circulating supply.

#### Bitcoin-only constant guards

The Bitcoin rune/degree systems use constants derived from `SUBSIDY_HALVING_INTERVAL`
that become zero with Dogecoin's value of 1.  Two guards prevent compile-time
division-by-zero panics in code that will never run on Dogecoin:

- `UNLOCK_INTERVAL` in `rune.rs` — guarded to minimum 1
- `HALVING_INCREMENT` in `sat.rs` — guarded to minimum 1

---

### 4. Inscription parsing (`src/inscriptions/envelope.rs`, `src/index/updater/inscription_updater.rs`)

**Critical difference**: Bitcoin inscriptions are embedded in Taproot witness data.
Dogecoin inscriptions are embedded in `input[0].script_sig`.

#### Parser (`RawEnvelope::from_transactions_dogecoin`)

```
scriptSig push ops:
  PUSH(<redeem script preamble>)   -- optional, ignored
  PUSH(b"ord")                     -- protocol marker
  PUSH(<tag byte>)                 -- field tag (e.g. [1] = ContentType)
  PUSH(<value>)                    -- field value
  PUSH([])                         -- empty tag = body separator
  PUSH(<body chunk>)               -- body data
  ...
```

The parser:
1. Iterates data pushes in `input[0].script_sig`
2. Finds the push equal to `b"ord"` (the protocol ID)
3. Treats all subsequent pushes as the envelope payload using the **standard ordinals
   tag-value format** — identical to Bitcoin's envelope payload layout, just in a
   different container

This means the existing `From<RawEnvelope> for ParsedEnvelope` conversion (which
handles field tags, body separator, etc.) works unchanged.

#### Dispatch (`inscription_updater.rs`)

```rust
let envelopes = if index.settings.chain().is_dogecoin() {
  ParsedEnvelope::from_transactions_dogecoin(std::slice::from_ref(tx))
} else {
  ParsedEnvelope::from_transaction(tx)
};
```

---

### 5. Small patches from apezord/ord-dogecoin

- **README** — Added `# Shibes` section at top with reorg checkpoint advice and a
  DOGE donation address
- **`src/index.rs`** — `get_block_by_height` / `get_block_by_hash` check the
  `HEIGHT_TO_BLOCK_HEADER` redb table before querying the RPC, so blocks that
  haven't been indexed yet return `None` rather than hitting the node
- **`src/lib.rs`** — Multi-Ctrl-C shutdown: pressing Ctrl-C more than
  `INTERRUPT_LIMIT` (5) times force-exits the process; a single press initiates a
  graceful shutdown

---

## Running against a Dogecoin node

```bash
# Index (starts from first_inscription_height = 4,600,000)
ord --chain dogecoin \
    --bitcoin-rpc-url http://127.0.0.1:22555 \
    --bitcoin-rpc-username <rpcuser> \
    --bitcoin-rpc-password <rpcpass> \
    index update

# Start the server
ord --chain dogecoin ... server --http
```

Your `dogecoin.conf` should have `txindex=1` enabled.  Keep your credentials in
a local config file (not committed to the repo).

---

## Files changed from upstream ord v0.25.0

| File | Change summary |
|---|---|
| `README.md` | Added Shibes section and DOGE donation address |
| `subsidies.json` | **New** — wonky-era per-block subsidies (145k entries) |
| `starting_sats.json` | **New** — cumulative sat totals at each block boundary (145k entries) |
| `src/chain.rs` | Dogecoin chain variants, genesis blocks, address encoding |
| `src/settings.rs` | Dogecoin data dir, cookie path, RPC chain detection |
| `src/index.rs` | Indexed-block guards |
| `src/lib.rs` | Multi-Ctrl-C shutdown |
| `src/inscriptions/envelope.rs` | `from_transactions_dogecoin` scriptSig parser |
| `src/index/updater/inscription_updater.rs` | Chain-aware inscription parser dispatch |
| `src/subcommand/epochs.rs` | Updated to use `Epoch::all_starting_sats()` |
| `src/subcommand/wallet/inscriptions.rs` | Added Dogecoin explorer URL arms |
| `crates/ordinals/Cargo.toml` | Added `serde_json` dependency |
| `crates/ordinals/src/lib.rs` | Dogecoin `SUBSIDY_HALVING_INTERVAL` / `DIFFCHANGE_INTERVAL` |
| `crates/ordinals/src/epoch.rs` | Full Dogecoin epoch/subsidy implementation |
| `crates/ordinals/src/sat.rs` | Updated `SUPPLY` ceiling; `HALVING_INCREMENT` guard |
| `crates/ordinals/src/rune.rs` | `UNLOCK_INTERVAL` guard |
