# Dog vs wonky-dogeord: Why Dog Is the Superior Dogecoin Indexer

**Dog (`jonheaven/dog`) is the clear winner**. It includes everything that works in **wonky-dogeord** (`tylordius/wonky-dogeord`), but better, faster, lighter, and with significantly more features.

Dog is actively maintained (commits as recent as hours ago), while wonky-dogeord has been dormant since October 2024.

## Quick Side-by-Side Comparison

| Feature / Capability | wonky-dogeord (tylordius) | dog (jonheaven) | Winner & Why |
|---|---|---|---|
| **Base** | Early fork of `apezord/ord-dogecoin` | Direct port of `ord v0.25.0` + heavy Dogecoin-specific upgrades | **dog** (newer ord base) |
| **Inscriptions / Doginals** | Yes (from height 4,609,723, scriptSig envelopes) | Yes (full Doginals v1 spec, multi-part, envelope parser) | **dog** (more complete parser + `dog scan`) |
| **Dunes** | Yes (from height 5,084,000) | Yes (`--index-dunes`) | Tie (dog has selective control) |
| **DRC-20 tokens + balances/holders** | Yes (requires `--index-transactions`) | Yes (full CLI: `drc20 tokens`, `drc20 balance`, mint/deploy support) | **dog** (dedicated CLI + address indexing) |
| **Block rewards / subsidies** | "Wonky" fix in `epoch.rs` for blocks 0-144,999 | Accurate Dogecoin subsidies via `subsidies.json` + `starting_koinu.json` | **dog** (external JSON is easier to audit/update) |
| **Indexing method** | RPC only + `--nr-parallel-requests` | **Direct `.blk` file reads** (5-20x faster) + shadow blk-index | **dog** (huge win) |
| **Selective / lightweight indexing** | Limited | Granular: `--only dogemap`, `--no-index-inscriptions`, `--index-koinu`, `--index-addresses`, etc. | **dog** (tiny protocol-specific nodes possible) |
| **Extra Dogecoin-native protocols** | None | **Dogemaps** + **.doge DNS** | **dog** |
| **Koinu / sat tracking** | No | Full koinu ranges + relic support | **dog** |
| **CLI tools & wallet** | Basic `ord` commands | Rich CLI: `scan`, `inscribe --dogemap`, `dns`, `drc20`, `dogemap status/list` + wallet | **dog** |
| **Web explorer / server** | Basic server + OpenAPI spec | Full web UI + API endpoints (`/dogemap/{block}` SVG, etc.) | **dog** |
| **Scan without full index** | No | `dog scan --from X --to Y --out` | **dog** |
| **Storage / performance** | ~400 GB full index | More efficient (selective + `.blk`) + redb with reorg backups | **dog** |
| **Docker / deployment** | Yes + docker-compose | Yes (Dockerfile) + quickstart scripts | Tie |
| **Activity** | 21 commits, last Oct 2024 | 1,670+ commits, 145 contributors, daily updates | **dog** |

## Does dog have everything that works in wonky-dogeord?

**Yes, and improved**:
- Doginals indexing: yes (more robust parser + scan tool)
- Dunes: yes
- DRC-20 (including holders): yes (via `--index-addresses` + dedicated commands)
- Accurate early block rewards: yes (`subsidies.json` + koinu handling)
- Server/API: yes (plus improved UI and Dogemap SVG endpoints)

## Why dog is strictly better

- **Speed**: Direct `.blk` parsing bypasses RPC for much faster initial sync.
- **Flexibility**: Selective indexing avoids unnecessary full-chain heavy indexing.
- **Dogecoin-first features**: Dogemaps and `.doge` DNS are native ecosystem features not present in wonky.
- **Modern ord base**: Includes newer ord stability and reorg handling work.
- **CLI-first workflow**: Full wallet/inscribe and explorer-friendly tooling.

## Minor things wonky has that dog handles differently

- Explicit OpenAPI YAML file: dog documents server behavior in `docs/` and live deployments.
- Explicit "wonky rewards" comment in `epoch.rs`: dog handles this through data-driven JSON subsidy tables.

## Verdict

**Use dog.** It is the actively developed Doginals indexer and explorer. wonky-dogeord was an important early step, but is now primarily useful as historical reference.

## Quick Start

```bash
git clone https://github.com/jonheaven/dog.git
cd dog
cargo build --release
# then follow the quickstart in README.md
```
