# Dog vs ord-dogecoin: Why Dog Is the Superior Dogecoin Indexer

**Dog (`jonheaven/dog`) is the clear winner**. It includes **everything** that actually works in **ord-dogecoin (apezord)**, but **better, faster, lighter, and with a ton more features**.

`ord-dogecoin` has been completely dormant since March 2023 (last commit: "only return blocks once indexed"). Dog is actively maintained with commits as recent as **hours ago** (March 2026) and 145 contributors.

## Quick Side-by-Side Comparison

| Feature / Capability | ord-dogecoin (apezord) | dog (jonheaven) | Winner & Why |
|---|---|---|---|
| **Base** | Early 2023 fork of original Bitcoin `ord` | Direct modern port of latest `ord` + full Dogecoin upgrades | **dog** (current ord stability + reorg handling) |
| **Inscriptions / Doginals** | Basic (scriptSig envelopes from ~height 4.6M) | Full Doginals v1 spec (multi-part, envelope parser, koinu math) | **dog** (complete parser + `dog scan` tool) |
| **Dunes** | No | Yes (`--index-dunes`) | **dog** |
| **DRC-20 tokens + balances/holders** | No | Yes (full CLI: `drc20 tokens`, `drc20 balance`, mint/deploy) | **dog** (dedicated commands + address indexing) |
| **Block rewards / subsidies** | Basic (no special handling) | Accurate via `subsidies.json` + `starting_koinu.json` | **dog** (clean and auditable) |
| **Indexing method** | RPC only (dogecoind + txindex) | **Direct .blk file reads** (5-20x faster) + shadow blk-index | **dog** (huge win) |
| **Selective / lightweight indexing** | None (all-or-nothing) | Extremely granular: `--only dogemap`, `--index-koinu`, `--no-index-inscriptions`, etc. | **dog** (run a tiny Dogemap-only node) |
| **Extra Dogecoin-native protocols** | None | **Dogemaps** (permanent block titles + procedural SVG API), **.doge DNS** (resolve/list/config) | **dog** (community staples) |
| **Koinu / sat tracking** | No | Full koinu ranges + relic support | **dog** |
| **CLI tools & wallet** | Basic `ord` wallet commands | Rich CLI: `scan`, `inscribe --dogemap`, `dns`, `drc20`, `dogemap status/list` + full wallet | **dog** |
| **Web explorer / server** | Basic server | Full web UI + API endpoints (`/dogemap/{block}` SVG etc.) | **dog** |
| **Scan without full index** | No | `dog scan --from X --to Y --out` (exports to disk) | **dog** |
| **Reorg handling** | Manual redb checkpoints required | Built-in redb reorg backups + shadow index | **dog** (no manual work) |
| **Storage / performance** | Full index only, RPC-limited | Selective + direct .blk = dramatically lighter and faster | **dog** |
| **Docker / deployment** | None mentioned | Yes (Dockerfile) + quickstart scripts | **dog** |
| **Activity** | 0 commits since Mar 2023 (46 stars, dormant) | 1,670+ commits, 145 contributors, daily updates | **dog** |

## Does dog have everything that works in ord-dogecoin?

**Yes: 100% covered, and dramatically improved**:
- Basic Doginals/inscriptions indexing: yes (far more robust parser + `dog scan`)
- RPC syncing with Dogecoin Core: yes (plus optional `.blk` bypass)
- Basic server/explorer: yes (much nicer UI + Dogemap SVG endpoints)
- Wallet commands: yes (safer defaults + modern UX)
- Reorg awareness: yes (automated, no manual checkpoints needed)

### Why dog is strictly better

- **Speed**: Direct `.blk` parsing bypasses RPC for much faster initial sync.
- **Flexibility**: Selective indexing means you do not index what you do not need.
- **Dogecoin-first features**: Dogemaps, `.doge` DNS, koinu tracking, DRC-20, and Dunes are all present in dog.
- **Modern ord base**: Pulls in years of upstream stability, reorg fixes, and database improvements.
- **Maintenance and community**: Actively developed vs. dormant since 2023.

### Minor things ord-dogecoin has that dog does not (and why it does not matter)

- Donation addresses in README.
- Basic reorg warning text instead of automated handling.

## Verdict

**Use dog.** It is the current, actively developed, official Doginals indexer and explorer for Dogecoin. `ord-dogecoin` (apezord) was the pioneering fork in 2023 that proved the concept, but it is now obsolete for real-world use.

## Quick Start

```bash
git clone https://github.com/jonheaven/dog.git
cd dog
cargo build --release
# then follow the quickstart in README.md (copy .env.example, point to your Dogecoin data dir, etc.)
```
