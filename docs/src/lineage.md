# Lineage & State of the Art

This document explains where `dog` came from, what was built on top of each
prior implementation, and why this repository represents the most technically
advanced Doginals implementation in existence.

---

## The Three Generations of Doginals

| Codebase | Based on | Status |
|---|---|---|
| [apezord/ord-dogecoin](https://github.com/apezord/ord-dogecoin) | ord ~v0.5 (2022) | Abandoned March 2023 |
| [verydogelabs/wonky-ord-dogecoin](https://github.com/verydogelabs/wonky-ord-dogecoin) | ord ~v0.5 (2022) | Stale since Oct 2024 |
| **this repo (`dog`)** | **ord v0.25.0 (latest)** | **Active** |

Both prior community forks were built on Bitcoin's `ord` ~v0.5 — roughly
**20 major versions** behind the upstream `ord v0.25.0` that this repo is
based on. Everything that Bitcoin's ordinals ecosystem learned and built
between 2022 and 2025 is present here; none of it existed in the older forks.

---

## What Was Inherited From Each Fork

### From apezord/ord-dogecoin

apezord was the pioneer — the first person to make ordinals work on Dogecoin
at all. The key insight he contributed to the ecosystem was:

- **Inscriptions live in `scriptSig`, not witness data.** Bitcoin inscriptions
  use Taproot witness data. Dogecoin has no Taproot, so inscriptions are
  embedded in `input[0].script_sig` using push opcodes. This fundamental
  difference is what makes Doginals technically distinct from Bitcoin
  Ordinals at the protocol level.
- Basic Dogecoin chain configuration (RPC ports, genesis block awareness)
- The `b"ord"` protocol marker — the three bytes that identify a Doginal
  inscription on-chain. **This has not changed and must never change**, as
  all existing Doginals on the blockchain use it.

### From verydogelabs/wonky-ord-dogecoin

The wonky team tackled the hardest purely-Dogecoin problem: **the wonky era**.

Blocks 0–144,999 of the Dogecoin blockchain each received a *random* block
reward between 0 and 1,000,000 DOGE. This makes it impossible to calculate
the cumulative koinu count for any block in that range using a formula — you
have to know the actual reward for every single block.

The wonky team assembled the community-accepted ground truth for this data:

- **`subsidies.json`** — the per-block reward in shibs (koinu) for all
  145,000 wonky-era blocks
- **`starting_sats.json`** — cumulative koinu totals at each block boundary

These files are embedded at compile time and are the canonical source of
truth for Doginal koinu numbering. Every indexer that correctly numbers
Doginals uses these values.

---

## What Is New in This Repo

Neither prior fork was ever updated beyond ord ~v0.5. This repo starts fresh
from **ord v0.25.0** and adds everything needed to make it work for Dogecoin.
That means the entire Bitcoin ordinals ecosystem's progress from 2022 to 2025
is now available for Doginals:

### Technical improvements over both prior forks

| Feature | apezord | wonky | **dog** |
|---|---|---|---|
| ord base version | ~v0.5 | ~v0.5 | **v0.25.0** |
| Correct koinu numbering | Partial | Yes | **Yes** |
| scriptSig inscription parser | Yes | Yes | **Yes (rewritten)** |
| Dogecoin address encoding | Basic | Basic | **Full P2PKH + P2SH** |
| Dunes (Dogecoin Runes) protocol | No | No | **Stubbed & ready** |
| Wallet subcommands | Basic | Basic | **Full (inscribe, send, receive, batch, burn, split, sweep…)** |
| Explorer / server | Basic | Basic | **Full with collections, galleries, attributes** |
| JSON API | Partial | Partial | **Complete** |
| Index addresses | No | No | **Yes** |
| Index transactions | No | No | **Yes** |
| Savepoints / reorg recovery | No | No | **Yes** |
| RSS feed | No | No | **Yes** |
| Maintained / compiling | No | No | **Yes** |

### Protocol-level correctness

The inscription parser was rewritten from scratch to match the full ord v0.25.0
envelope format (tag-value fields, body separator, delegate support, metadata,
pointer, parent/child relationships) — all inside the Dogecoin `scriptSig`
container rather than Bitcoin's taproot witness.

### Full Dogecoin rebrand

The codebase was fully renamed to reflect Dogecoin's own terminology:

- Binary: `ord` → `dog`
- Runes → **Dunes** (Dogecoin's equivalent protocol)
- Satoshis/Sats → **Koinu** (Dogecoin's smallest unit)
- Ordinals → **Doginals**
- Bitcoin chains removed; only `dogecoin`, `dogecoin-testnet`,
  `dogecoin-regtest` remain

---

## Why This Matters

The prior forks were built on a codebase that is now years out of date. They
lack features that the Bitcoin ordinals community has come to rely on, and they
have not received the bug fixes and security improvements that have gone into
upstream `ord` since 2022.

This repository is the **only** Doginals implementation that:

1. Is based on current upstream ordinals technology
2. Correctly handles the full Dogecoin subsidy/koinu numbering
3. Has a complete, modern explorer and wallet
4. Compiles and runs against a current Dogecoin Core node
5. Is actively maintained

If you are building Doginals tooling, indexers, wallets, or explorers, this
is the correct foundation to build on.
