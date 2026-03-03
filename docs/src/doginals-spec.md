# Official Doginals Protocol v1

**Dogecoin Inscriptions (Doginals)** — March 2026
**Canonical reference maintained by [jonheaven/dog](https://github.com/jonheaven/dog)** (the fastest and most correct Dogecoin Ordinals indexer)

Doginals give individual **koinu** (1 DOGE = 100,000,000 koinu) numismatic value by
permanently binding arbitrary data to them via on-chain inscriptions.

This is the **only** official specification. Any indexer or marketplace that deviates
is incompatible.

---

## 1. Koinu Numbering (The Source of Rarity)

Dogecoin does **not** halve to zero. The subsidy schedule is irregular at the beginning
and becomes fixed at **10,000 DOGE per block forever** after block 600,000.

The canonical indexer ships two reference files:

- [`starting_koinu.json`](../../../starting_koinu.json) — cumulative koinu offset before each block
- [`subsidies.json`](../../../subsidies.json) — per-height subsidy, generated from Dogecoin Core's
  `GetBlockSubsidy` (`dogecoin/src/validation.cpp`)

### Numbering rules

- Every koinu is numbered sequentially from genesis (block 0, first koinu = 0).
- The first koinu of block `N` = `starting_koinu[N]`.
- The last koinu of block `N` = `starting_koinu[N] + subsidy(N) − 1`.
- Rarity tiers are computed from these numbers using the indexer's rarity engine.

This is the **only** correct method. Any other approach will be off by millions of koinu
after block 600,000.

### Subsidy schedule (post-wonky era)

| Block range        | Reward per block |
|--------------------|-----------------|
| 0 – ~145,000       | Irregular (wonky era — see `subsidies.json`) |
| 145,000 – 199,999  | 500,000 DOGE    |
| 200,000 – 299,999  | 250,000 DOGE    |
| 300,000 – 399,999  | 125,000 DOGE    |
| 400,000 – 499,999  | 62,500 DOGE     |
| 500,000 – 599,999  | 31,250 DOGE     |
| 600,000+           | **10,000 DOGE (permanent floor — no final halving)** |

### Rarity tiers

| Tier       | Condition                                      |
|------------|------------------------------------------------|
| mythic     | First koinu of the genesis block (1 total)     |
| legendary  | First koinu of any block                       |
| epic       | Last koinu of any block                        |
| rare       | First koinu after the 10 % mark of a block     |
| uncommon   | First koinu after the 50 % mark of a block     |
| common     | Everything else (~99.999 %)                    |

---

## 2. Inscription Envelope (Legacy Pushdata Format)

Dogecoin has **no Taproot and no SegWit** as of 2026. All Doginals use the
**legacy pushdata envelope** embedded in `input[0].script_sig` of the reveal
transaction — the same format that apezord pioneered and that every surviving
Doginal uses today.

### scriptSig push sequence

```text
PUSH("ord")              ← 3-byte protocol marker  (0x03 0x6f 0x72 0x64)
PUSH(<tag>)              ← field tag byte(s)
PUSH(<value>)            ← field value
... (additional tag / value pairs)
PUSH("")                 ← empty push at a tag position = body separator
PUSH(<body_chunk>)       ← content bytes
[ PUSH(<body_chunk>) ]   ← repeat for large payloads
```

The parser (`src/inscriptions/envelope.rs` → `from_transactions_dogecoin()`) collects
every data-push from `input[0].script_sig`, finds the first push equal to `"ord"`,
and treats all subsequent pushes as the tag-value payload.

### Field tags

| Tag (hex) | Field              | Notes                                        |
|-----------|--------------------|----------------------------------------------|
| `01`      | `content_type`     | MIME string, e.g. `text/plain;charset=utf-8` |
| `02`      | `pointer`          | koinu-offset redirect                        |
| `03`      | `parent`           | parent inscription ID                        |
| `05`      | `metadata`         | CBOR metadata blob                           |
| `07`      | `metaprotocol`     | sub-protocol label (e.g. `drc-20`, `dns`)    |
| `09`      | `content_encoding` | `br`, `gzip`, etc.                           |
| `0b`      | `delegate`         | delegation target inscription ID             |
| `00`      | *(body separator)* | empty push at an even (tag) position         |

Even-numbered tags are consensus-critical; unknown odd tags are tolerated (same
rule as Bitcoin Ordinals).

### Minimal example — a plain-text inscription

```
scriptSig pushes (in order):
  0x6f7264                     ← "ord"
  0x01                         ← tag: content_type
  "text/plain;charset=utf-8"   ← value
  ""                           ← body separator (empty push at even position)
  "Hello, Dogecoin!"           ← body content
```

### Multi-part inscriptions

Content too large for a single transaction is split across multiple transactions.
The push immediately after `"ord"` is the piece count (a push integer). `dog`
reassembles parts by scanning for matching continuation transactions within the
same block range.

---

## 3. Inscription ID Format

```
<txid>i<index>
```

- `txid` — reveal transaction hash (little-endian hex, same as Dogecoin Core)
- `i<index>` — zero-based input index of the envelope (`i0` for first input)

**Example:**

```
bdfeeeacab95d0a230e1124f0635ac9a47925fef4bb1d41a0a0c6e8d8232af7ai0
```

This ID is permanent and immutable. Every marketplace, wallet, and explorer must
use this format.

---

## 4. Binding and Transfer

1. The inscription is bound to the first koinu of the output associated with the
   envelope transaction.
2. When the UTXO is spent, the inscription moves with the first koinu of the input
   (ordinal theory tracking).
3. The `dog` wallet and indexer track provenance automatically.

---

## 5. Built-in Extensions

### DRC-20 tokens

Deploy, mint, and transfer fungible tokens using JSON inscription content with
`metaprotocol = "drc-20"`:

```json
{ "p": "drc-20", "op": "deploy",   "tick": "dogi", "max": "21000000", "lim": "1000" }
{ "p": "drc-20", "op": "mint",     "tick": "dogi", "amt": "1000" }
{ "p": "drc-20", "op": "transfer", "tick": "dogi", "amt": "500" }
```

See [`drc20.md`](drc20.md) and `dog drc20` commands.

### Dogecoin Name System (DNS)

`.doge` names stored as inscriptions and resolved by:

```
dog dns resolve satoshi.doge
```

See [`dns.md`](dns.md).

---

## 6. Dogecoin-Specific Notes

- **AuxPoW** merged-mining headers (after block 371,337) are fully supported in the
  `.blk` parser — a naive Bitcoin fork would choke here.
- **No SegWit discount** — inscription size limits follow standard Dogecoin transaction
  data limits.
- **Scrypt PoW** — difficulty and 1-minute block times are handled by Dogecoin Core
  running alongside the indexer.
- **Addresses** — all addresses use Dogecoin base58check encoding (P2PKH prefix 0x1e
  → `"D..."`, P2SH prefix 0x16 → `"A..."`), HD coin type 3.
- **Network magic bytes** — mainnet `0xC0C0C0C0`, testnet `0xFCC1B7DC`,
  regtest `0xFABFB5DA` (see `src/chain.rs` `chainparams` module).

---

## 7. Indexer Compatibility Requirements

Any indexer claiming Doginals compatibility **must**:

1. Use `starting_koinu.json` + `subsidies.json` exactly as shipped in this repo.
2. Parse the legacy pushdata envelope above (no Taproot/SegWit assumptions).
3. Assign inscription IDs in `<txid>i<index>` format.
4. Handle Dogecoin AuxPoW block headers after block 371,337.
5. Respect Dogecoin reorg depth (1-minute blocks have higher reorg frequency than Bitcoin).

---

## 8. Future Extensions (v2 Candidates)

- Recursive inscriptions
- On-chain collection metadata
- Richer royalty encoding
- Marketplace orderbook protocol

The v1 envelope format and koinu numbering are **final and will not change**.

---

## Reference Implementation

**[jonheaven/dog](https://github.com/jonheaven/dog)** — the official Doginals indexer,
parser, wallet, and block explorer.

```
# Scan any block range without a full index
dog scan --from 4609000 --to 4700000

# Full index + explorer
dog index update && dog server
```

This specification is binding. All marketplaces, explorers, wallets, and AI agents
building on Dogecoin inscriptions must follow it to remain compatible.

---

*Made with ❤️ for the Dogecoin community.*
*jonheaven/dog team — March 2026*
