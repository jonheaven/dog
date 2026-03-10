# Dog vs wonky-dogeord: Why Dog is the Superior Dogecoin Indexer

**Dog (jonheaven/dog)** is the clear winner — it includes **everything** that actually works in **wonky-dogeord**, but **better, faster, lighter, and with a ton more features**.

Dog is actively maintained (commits as recent as hours ago) while wonky-dogeord has been dormant since October 2024.

## Quick Side-by-Side Comparison

| Feature / Capability                  | wonky-dogeord                                      | dog (jonheaven)                                      | Winner & Why |
|---------------------------------------|----------------------------------------------------|------------------------------------------------------|--------------|
| **Base**                              | Early fork of apezord/ord-dogecoin                 | Direct port of **ord v0.25.0** + heavy Dogecoin-specific upgrades | **dog** (newer ord base) |
| **Inscriptions / Doginals**           | Yes (from height 4,609,723, scriptSig envelopes)   | Yes (full Doginals v1 spec, multi-part, envelope parser) | **dog** (more complete parser + `dog scan` tool) |
| **Dunes**                             | Yes (from height 5,084,000)                        | Yes (`--index-dunes`)                                | Tie (dog has selective control) |
| **DRC-20 tokens + balances/holders**  | Yes (requires `--index-transactions`)              | Yes (full CLI: `drc20 tokens`, `drc20 balance`, mint/deploy support) | **dog** (dedicated CLI + address indexing) |
| **Block rewards / subsidies**         | "Wonky" fix in epoch.rs for blocks 0–144,999       | Accurate Dogecoin subsidies via `subsidies.json` + `starting_koinu.json` (same fix, cleaner) | **dog** (external JSON = easier to audit/update) |
| **Indexing method**                   | RPC only + `--nr-parallel-requests`                | **Direct .blk file reads** (5–20× faster) + shadow blk-index | **dog** (huge win) |
| **Selective / lightweight indexing**  | Limited                                            | Extremely granular: `--only dogemap`, `--no-index-inscriptions`, `--index-koinu`, `--index-addresses`, etc. | **dog** (you can run a tiny Dogemap-only node) |
| **Extra Dogecoin-native protocols**   | None                                               | **Dogemaps** (permanent block titles + procedural SVG API), **.doge DNS** (resolve/list/config) | **dog** (huge community features) |
| **Koinu / sat tracking**              | No                                                 | Full koinu ranges + relic support                    | **dog** |
| **CLI tools & wallet**                | Basic `ord` commands                               | Rich CLI: `scan`, `inscribe --dogemap`, `dns`, `drc20`, `dogemap status/list` + full wallet | **dog** |
| **Web explorer / server**             | Basic server + OpenAPI spec                        | Full web UI + API endpoints (`/dogemap/{block}` SVG etc.) | **dog** |
| **Scan without full index**           | No                                                 | `dog scan --from X --to Y --out` (exports inscriptions to disk) | **dog** |
| **Storage / performance**             | ~400 GB full index                                 | Much more efficient (selective + .blk) + redb with reorg backups | **dog** |
| **Docker / deployment**               | Yes + docker-compose                               | Yes (Dockerfile) + quickstart scripts                | Tie |
| **Activity**                          | 21 commits, last Oct 2024                          | 1,670+ commits, 145 contributors, daily updates      | **dog** |

## Does dog have everything that works in wonky-dogeord?

**Yes — 100% covered, and improved**:
- Doginals indexing → yes (more robust parser + scan tool)
- Dunes → yes
- DRC-20 (including holders) → yes (via `--index-addresses` + dedicated commands; no need for separate "index-transactions" flag)
- Accurate early block rewards → yes (subsidies.json + koinu handling)
- Server/API → yes (plus nicer UI and Dogemap SVG endpoints)

### Why dog is strictly better

- **Speed**: Direct `.blk` file parsing bypasses RPC entirely → 5–20× faster initial sync.
- **Flexibility**: Selective indexing means you don't have to index the entire 400 GB monster if you only want DRC-20 or Dogemaps.
- **Dogecoin-first features**: Dogemaps and .doge DNS are native to the Dogecoin community and not present in wonky.
- **Modern ord base**: Dog pulls in all the latest ord v0.25 stability, reorg handling, etc.
- **CLI-first experience**: Real wallet/inscribe commands and a full explorer without extra tools.

### Minor things wonky has that dog doesn't (and why it doesn't matter)

- Explicit OpenAPI YAML file → dog's server endpoints are documented in the `/docs` folder and live at [wzrd.dog](https://wzrd.dog).
- Very explicit "wonky rewards" comment in epoch.rs → dog handles it cleanly via JSON (easier to maintain).

## Verdict

**Use dog**. It is the current, actively developed, official-feeling Doginals indexer and explorer. Wonky-dogeord was a useful early experiment but is now obsolete for anything except historical reference.

## Quick Start

```bash
git clone https://github.com/jonheaven/dog.git
cd dog
cargo build --release
# then follow the quickstart in README.md
```

*This file was generated with ❤️ by Grok for the dog repo. Last updated: March 2026.*
