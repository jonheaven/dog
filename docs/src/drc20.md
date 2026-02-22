# DRC-20 Tokens

DRC-20 is a fungible token standard on Dogecoin, implemented as JSON-encoded
Doginal inscriptions. `dog` indexes deploy, mint, and transfer operations
automatically during `dog index update` and exposes them through the
`dog drc20` subcommands.

## Token lifecycle

### Deploy

A deployer inscribes a JSON object to create a new token:

```json
{
  "p": "drc-20",
  "op": "deploy",
  "tick": "dogi",
  "max": "21000000",
  "lim": "1000",
  "dec": "8"
}
```

| Field | Description |
|-------|-------------|
| `tick` | 4-character ticker symbol (case-insensitive) |
| `max`  | Maximum total supply |
| `lim`  | Maximum amount per mint operation |
| `dec`  | Decimal places (default 18, max 18) |

### Mint

Anyone can mint up to `lim` tokens per inscription:

```json
{
  "p": "drc-20",
  "op": "mint",
  "tick": "dogi",
  "amt": "1000"
}
```

Mints beyond the `max` supply or `lim` per-mint cap are silently ignored.

### Transfer

Transfers happen in two steps:

1. **Inscribe** a transfer inscription to your address:

```json
{
  "p": "drc-20",
  "op": "transfer",
  "tick": "dogi",
  "amt": "500"
}
```

2. **Send** the inscription output to the recipient. The balance moves when the
   inscription UTXO is spent.

## Commands

### List all tokens

```sh
dog drc20 tokens
```

Output columns: ticker, max supply, minted, mint limit, decimals, holders,
deploy height.

```sh
dog drc20 tokens --json
```

### Show a single token

```sh
dog drc20 token dogi
```

```sh
dog drc20 token dogi --json
```

Fields returned: tick, max_supply, mint_limit, decimals, minted, deploy
inscription ID, deploy height, deploy timestamp, deployer address, mint count.

### Show balances for an address

```sh
# All tokens held by an address
dog drc20 balance DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc

# Single token
dog drc20 balance DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc --tick dogi

# JSON
dog drc20 balance DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc --json
```

## Indexing requirement

DRC-20 data is stored in the redb index. You must run `dog index update` before
querying balances or token info. The `dog drc20` commands will return an error
if the index has not been initialized.

## Notes

- Ticker comparisons are case-insensitive (`DOGI` == `dogi`).
- Amounts are stored as fixed-point integers scaled by `10^decimals` to avoid
  floating-point precision loss.
- Only the first valid deploy inscription for a given ticker is accepted;
  subsequent deploys for the same tick are ignored.
- Transfer inscriptions only credit the recipient when the inscription UTXO is
  actually spent to that address.
