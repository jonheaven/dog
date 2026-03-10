# Dog vs wonky-dogeord: Why Dog Is the Superior Dogecoin Indexer

**Dog (`jonheaven/dog`) is the clear winner**. It includes everything that works in **wonky-dogeord** (`tylordius/wonky-dogeord`), but better, faster, lighter, and with significantly more features.

Dog is actively maintained (commits as recent as hours ago), while wonky-dogeord has been dormant since October 2024.

## Quick Side-by-Side Comparison

### At-a-Glance Result

- `dog` wins: **12** categories
- Tie: **2** categories
- `wonky-dogeord` wins: **0** categories

### Protocol Coverage

| Capability | wonky-dogeord | dog | Verdict |
|---|---|---|---|
| Inscriptions / Doginals | Supported | Supported (full v1 parser + `dog scan`) | **dog** |
| Dunes | Supported | Supported (`--index-dunes`) | Tie |
| DRC-20 tokens and balances | Supported (`--index-transactions`) | Supported (dedicated `drc20` CLI + address indexing) | **dog** |
| Early subsidy handling | Hardcoded fix in `epoch.rs` | Data-driven `subsidies.json` + `starting_koinu.json` | **dog** |

### Performance and Indexing

| Capability | wonky-dogeord | dog | Verdict |
|---|---|---|---|
| Sync architecture | RPC-driven indexing | Direct `.blk` reads + shadow block index | **dog** |
| Initial sync speed | Slower (RPC bottleneck) | Typically much faster (5-20x) | **dog** |
| Selective indexing | Limited | Granular flags (`--only`, `--no-index-inscriptions`, etc.) | **dog** |
| Storage profile | Heavy full index (~400 GB) | Leaner options via selective indexing | **dog** |

### Product and UX

| Capability | wonky-dogeord | dog | Verdict |
|---|---|---|---|
| Koinu tracking | Not first-class | Full koinu range + relic support | **dog** |
| CLI experience | Basic ord-like commands | Rich Dogecoin-specific command surface | **dog** |
| Explorer/API | Basic server + OpenAPI file | Full web UI + Dogemap SVG/API endpoints | **dog** |
| Scan without full index | Not available | `dog scan --from X --to Y --out` | **dog** |

### Dogecoin-Native Features

| Capability | wonky-dogeord | dog | Verdict |
|---|---|---|---|
| Dogemaps protocol | No | Yes | **dog** |
| `.doge` DNS tools | No | Yes (`dns resolve/list/config`) | **dog** |
| Deployment options | Docker + compose | Dockerfile + quickstart scripts | Tie |
| Maintenance activity | Inactive since Oct 2024 | Active daily development | **dog** |

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
