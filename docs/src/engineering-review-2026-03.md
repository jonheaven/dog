# Doginals Engineering Review (2026-03)

This document records the findings from an engineering audit of the `dog` indexer in
March 2026.  It distinguishes between **changes already shipped** and **open
recommendations** that have not yet been implemented.

---

## Status key

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and shipped |
| 🔲 | Recommended — not yet implemented |

---

## What is working well

- Dogecoin-specific chain parameters are explicit and easy to audit (`MAGIC_*`,
  genesis hashes, chain aliases) — consensus-sensitive behaviour is visible in one
  place.
- The project already has Dogecoin-native product features beyond Ordinals parity:
  DNS, DRC-20, Dogemaps, block-range scanning, and Dune token indexing.
- The indexer has a savepoint mechanism for reorg recovery; docs warn users about
  Dogecoin reorg characteristics.
- Fast `.blk` direct-read flow is implemented and documented as a first-class
  indexing mode.

---

## Shipped fixes (this audit cycle)

### ✅ Reorg recovery target selection (correctness fix)

`Reorg::handle_reorg` previously restored the **oldest** available savepoint.
On Dogecoin, frequent shallow reorgs are expected; restoring the oldest savepoint
could roll back much further than necessary, increasing downtime and replaying large
historical windows.

**What landed (`b1033356`):**
- Switch recovery to the **newest** available savepoint (`.max()` instead of `.min()`
  on the savepoint-id iterator).
- Return an explicit error (`"unable to recover from reorg: no savepoints available"`)
  instead of silently failing when no savepoints exist.

**Open follow-up (🔲):** Store savepoint metadata keyed by block height so
`handle_reorg` can select the best savepoint at or below `height - reorg_depth`
rather than always picking the single newest one.

---

### ✅ `/health` endpoint

A lightweight `GET /health` JSON endpoint was added to support production monitoring without
parsing the full `/status` response.

**What landed:**
- `Index::health()` method calls `client.get_block_count()` (RPC) and compares to the
  indexed tip from the `HEIGHT_TO_BLOCK_HEADER` table.
- Returns `{ index_tip, chain_tip, lag_blocks, status }` where `status` is `"synced"`,
  `"syncing"` (≤6 blocks behind), or `"behind"` (>6 blocks behind).
- Route: `GET /health` → always JSON, no Accept header required.

---

### ✅ Dogemap rarity tiers

Every Dogemap block now carries a `rarity` field in its `/dogemap/{block}` JSON response.

**Tier definitions:**

| Tier | Condition |
|------|-----------|
| mythic | Block 0 (genesis) |
| legendary | Epoch-transition blocks (145,000 / 200,000 / 300,000 / 400,000 / 500,000 / 600,000) |
| epic | Any block divisible by 1,000 |
| rare | Any block divisible by 100 |
| uncommon | Any block divisible by 10 |
| common | Everything else |

---

### ✅ Metaverse JSON field on `/dogemap/{block}`

A `metaverse` object was added to every `/dogemap/{block}` response.  All fields are
deterministically derived from the block hash and transaction count — the same block always
produces the same values.

**Fields:**

| Field | Range | Description |
|-------|-------|-------------|
| `color_hue` | 0–359 | HSL hue for the block's primary colour |
| `elevation` | 0–255 | Terrain height seed |
| `terrain_seed` | u32 | 32-bit noise seed for procedural terrain |
| `activity` | 0–100 | Transaction density proxy (clamped) |
| `biome` | string | One of 8 biome themes driven by hash + block parity |

Biome values: `desert`, `tundra`, `jungle`, `ocean`, `volcanic`, `grassland`, `canyon`, `space`.

---

## Open recommendations

### 🔲 Make Dogecoin consensus constants source-generated

Dogecoin network constants (`MAGIC_*`, genesis hashes, ports) are currently
hardcoded in `src/chain.rs`.

- **Recommendation:** add a CI script that validates these constants against
  `dogecoin/src/chainparams.cpp` in the canonical Dogecoin Core repo.
- **Benefit:** prevents silent drift if upstream Dogecoin changes testnet/regtest
  defaults.

---

### 🔲 Reorg observability and SLO tracking

Reorg handling works but exposes no metrics for operators to monitor.

Suggested additions to `/status` or a dedicated `/health` endpoint:

```json
{
  "index_tip": 5056597,
  "chain_tip": 5056600,
  "lag_blocks": 3,
  "reorg_detected_total": 12,
  "reorg_recovered_total": 12,
  "reorg_unrecoverable_total": 0,
  "last_reorg_depth": 2,
  "savepoint_age_blocks": 5000
}
```

Counters should survive process restarts (persisted in redb).

---

### 🔲 RPC/load resilience for public explorer deployments

The code retries block fetches and can fall back from `.blk` reads to RPC.

- Add bounded, jittered exponential backoff on the RPC hot path.
- Add optional rate-limited worker pools for expensive endpoints (`scan`, `drc20`,
  DNS list).
- Add `Cache-Control` / `ETag` / `Last-Modified` for read-only explorer responses.

---

### 🔲 Compatibility matrix testing against Dogecoin Core releases

Given protocol-level coupling, add CI lanes for multiple Dogecoin Core versions
(latest release, previous stable, current master snapshot).

Smoke test scenarios:
- Initial sync from `.blk` files
- Reorg simulation + recovery
- Inscription parsing with legacy scriptSig envelopes
- DNS / DRC-20 / Dogemaps read paths

---

### 🔲 `dog index audit` command

A data integrity command to verify DB invariants:
- Height continuity (no gaps in `HEIGHT_TO_BLOCK_HEADER`)
- Satpoint ownership uniqueness
- Inscription linkage (every inscription ID maps to a known txid)
- Dogemap claim uniqueness (no block number claimed twice)

---

### 🔲 Operator production guide

Document recommended settings for production deployments:
- Savepoint sizing guidance (interval / count tradeoffs for reorg depth vs. disk usage)
- SSD I/O tuning
- Snapshot cadence
- Alerting thresholds (lag, reorg rate)

---

### 🔲 Incremental redb compaction

Long-running nodes accumulate tombstones and fragmentation in the redb file.
Provide a `dog index compact` command or scheduled auto-compaction workflow.

---

### 🔲 Canonical API schemas

Publish JSON schemas (or OpenAPI fragments) for Dogemap, DNS, and DRC-20 API
responses to improve integrator stability.

---

### 🔲 WebSocket / event stream

A lightweight WebSocket endpoint for real-time new-inscription, Dogemap-claim,
and chain-tip-update events — useful for marketplace frontends and metaverse
integrations.

---

### 🔲 Deterministic replay harness

A test harness that replays blocks from a known height to a target height and
asserts an identical DB state hash, enabling regression detection across refactors.

---

## Suggested milestones

### Milestone 1 — Reliability
- ✅ Land reorg savepoint selection fix
- ✅ Add `/health` endpoint for production monitoring
- 🔲 Add reorg metrics (counters, `/health` reorg fields)
- 🔲 Add reorg simulation integration test

### Milestone 2 — Operations
- 🔲 Ship production runbook + monitoring dashboards
- 🔲 Add `dog index audit` and scheduled integrity checks
- 🔲 Redb compaction workflow

### Milestone 3 — Ecosystem
- ✅ Dogemap rarity tiers
- ✅ Metaverse JSON on `/dogemap/{block}`
- 🔲 Freeze and publish API schemas
- 🔲 WebSocket / event stream for ecosystem apps
- 🔲 Deterministic replay harness
