# Doginals Engineering Review (2026-03)

This review focuses on reliability under Dogecoin-specific chain conditions (1-minute blocks, deeper/frequent reorgs), indexer correctness, explorer/API operability, and long-term maintainability.

## What is working well

- Dogecoin-specific chain parameters are explicit and easy to audit (`MAGIC_*`, genesis hashes, and chain aliases). This makes consensus-sensitive behavior visible in one place.
- The project already has Dogecoin-native product features beyond Ordinals parity (DNS, DRC-20, Dogemap, block-range scanning).
- The indexer has a savepoint concept for reorg recovery, and documents warn users about Dogecoin reorg characteristics.
- Fast `.blk` direct read flow is already implemented and documented as a first-class indexing mode.

## High-priority fixes and improvements

### 1) Reorg recovery target selection (correctness bug)

`Reorg::handle_reorg` should restore the **newest** available savepoint, not the oldest one.

- Why this matters: on Dogecoin, frequent shallow reorgs are expected. Restoring from the oldest savepoint can rollback much further than necessary, increasing downtime and potentially replaying large historical windows.
- Action taken in this patch: switch recovery to newest savepoint and fail with explicit error if no savepoint exists.
- Follow-up: store savepoint metadata by height (savepoint id → block height) to restore the best savepoint at or before `height - depth`.

### 2) Make Dogecoin consensus constants source-generated

Dogecoin network constants are currently hardcoded. That's fine initially, but brittle over time.

- Recommendation: add a small CI check/script that validates constants against the canonical Dogecoin Core values from `dogecoin/src/chainparams.cpp`.
- Benefit: prevents silent drift if upstream Dogecoin changes testnet/regtest defaults.

### 3) Harden reorg observability and SLO tracking

Reorg handling exists but should expose metrics that operators can monitor.

- Add metrics/counters for:
  - `reorg_detected_total`
  - `reorg_recovered_total`
  - `reorg_unrecoverable_total`
  - `reorg_depth_histogram`
  - `savepoint_age_blocks`
- Add explorer/admin endpoint for current index tip vs core tip and reorg health.

### 4) Improve RPC/load resilience for public explorer deployments

The code retries block fetches and can fall back from `.blk` reads to RPC. Great baseline.

- Recommendation:
  - Add bounded, jittered exponential backoff everywhere RPC is on the hot path.
  - Add optional rate-limited worker pools for expensive endpoints (`scan`, `drc20`, DNS list endpoints).
  - Add cache controls and ETag/Last-Modified for explorer responses.

### 5) Introduce compatibility matrix testing against Dogecoin Core releases

Given protocol-level coupling, add CI lanes for Dogecoin Core versions (e.g., latest release, previous LTS-ish, and current master snapshot when practical).

- Run scripted smoke tests:
  - initial sync from `.blk`
  - reorg simulation + recovery
  - inscription parsing with legacy scriptSig envelopes
  - DNS/DRC-20 read paths

## Medium-priority improvements

- Add a documented data integrity command (`dog index audit`) to verify DB invariants (height continuity, satpoint ownership uniqueness, inscription linkage).
- Add an explicit operator guide for production deployments (snapshot cadence, savepoint sizing guidance, SSD/I/O tuning, alerting thresholds).
- Consider incremental redb compaction/maintenance workflow for long-running nodes.
- Expand fuzzing around script parsing and envelope extraction for malformed scriptSig edge cases.

## Product roadmap opportunities

- Add canonical JSON schemas/versioning for Dogemap, DNS, and DRC-20 APIs to improve integrator stability.
- Add a lightweight websocket stream for new inscriptions/transfers and chain-tip updates.
- Build a deterministic replay harness for “index from height X to Y and compare expected state hashes”.

## Suggested next milestones

1. **Reliability milestone**
   - Land reorg savepoint selection fix (included here).
   - Add reorg metrics and health endpoint.
   - Add reorg simulation integration test.

2. **Operations milestone**
   - Ship production runbook + monitoring dashboards.
   - Add `dog index audit` and scheduled integrity checks.

3. **Ecosystem milestone**
   - Freeze and publish API schemas.
   - Add webhook/websocket feeds for ecosystem apps.

