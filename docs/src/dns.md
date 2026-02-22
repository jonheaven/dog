# Dogecoin Name System

The Dogecoin Name System (DNS) maps human-readable names like `satoshi.doge`
to addresses, URLs, or arbitrary configuration, encoded as Doginal inscriptions.
Names are resolved entirely on-chain — no external registry required.

## Supported namespaces

| Namespace | Example |
|-----------|---------|
| `.doge`   | `satoshi.doge` |

Other namespaces may appear in inscription data; `dog dns list` shows all
names that `dog` has indexed regardless of namespace.

## Commands

### Resolve a name

Returns the resolved value (address, URL, or raw config) for the name:

```sh
dog dns resolve satoshi.doge
```

```sh
dog dns resolve jon.doge --json
```

### List all registered names

```sh
dog dns list
```

Filter by namespace:

```sh
dog dns list --namespace doge
```

```sh
dog dns list --namespace doge --json
```

### Show DNS configuration

Returns the full key-value configuration stored in the inscription:

```sh
dog dns config satoshi.doge
```

Common config keys (depends on the inscription author):

| Key | Meaning |
|-----|---------|
| `address` | Primary Dogecoin address |
| `url`     | Website or social link |
| `avatar`  | Inscription ID of an avatar image |
| `content` | Arbitrary text content |

## Indexing requirement

DNS data is stored in the redb index. Run `dog index update` before using
`dog dns` commands. Names inscribed after your last index update will not
appear until you re-run `dog index update`.

## How names are stored on-chain

A DNS inscription is a plain-JSON Doginal with content type `application/json`
or `text/plain`. The `dog` indexer recognizes name registrations by the
inscription content structure and the address that owns the inscription UTXO.

The owner of the inscription UTXO is the authoritative holder of the name.
Transferring the inscription transfers the name.
