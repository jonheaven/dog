# Scanning

`dog scan` reads a block range for inscriptions **without requiring a full redb index**.
It uses direct `.blk` file reads when available (fast path) and falls back to RPC
automatically. This makes it useful for spot-checking, verifying a specific
inscription, or exporting a collection without indexing the entire chain.

## Basic usage

```sh
# Find all inscriptions between two heights
dog scan --from 4609000 --to 4620000

# JSON output
dog scan --from 4609000 --to 4620000 --json
```

## Verify a specific inscription

Pass any prefix of the txid (or the full txid) to `--txid`:

```sh
dog scan --from 4609000 --to 4700000 \
  --txid bdfeeeacab95d0a230e1124f0635ac9a47925fef4bb1d41a0a0c6e8d8232af7a
```

`dog` will print a match as soon as it finds a transaction whose txid starts
with the given string.

## Filter by address

Only show inscriptions whose first output went to a specific address:

```sh
dog scan --from 4609000 --to 5000000 \
  --address DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc
```

## Export to disk

Use `--out <dir>` to save each inscription's content and metadata:

```sh
dog scan --from 4609000 --to 5000000 \
  --address DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc \
  --out ./my-inscriptions
```

The output directory structure is:

```
<out>/
  <height>_<txid>i<n>/
    content.<ext>    ← the actual image / text / audio
    info.json        ← height, txid, content_type, recipient, size
```

`info.json` example:

```json
{
  "inscription_id": "bdfeeeacab95d0a230e1124f0635ac9a47925fef4bb1d41a0a0c6e8d8232af7ai0",
  "height": 4609123,
  "txid": "bdfeeeacab95d0a230e1124f0635ac9a47925fef4bb1d41a0a0c6e8d8232af7a",
  "index": 0,
  "content_type": "image/png",
  "content_size": 42381,
  "recipient": "DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc"
}
```

## Combining filters

All flags can be combined:

```sh
# Export only PNG inscriptions sent to an address in a range, as JSON
dog scan --from 4609000 --to 4700000 \
  --address DHrqn6H6ocgbRB1Szu7Q1sn1tVTfkpinnc \
  --out ./wizard-dogs \
  --json
```

## Speed

`dog scan` uses the same direct `.blk` file reader as `dog index update`.
If you have run `dog index refresh-blk-index` (or it has been run automatically
during a previous `dog index update`), scanning reads blocks at full disk speed
without any RPC overhead.

That shadow copy now lives alongside the active Dogecoin Core data directory
at `<DOGECOIN_DATA_DIR>/<network>/blk-index/`, so `dog` and `kabosu` can share
the same LevelDB copy instead of maintaining separate copies on different
drives.

If the blk-index copy is not available, `dog scan` falls back to Dogecoin Core
RPC automatically — this works but is slower for large ranges.

See [Reindexing](guides/reindexing.md) for notes on the blk-index shadow copy.

## Progress output

Every 1,000 blocks `dog scan` prints a progress line to stderr:

```
  scanned through block 4610000 (14 inscriptions so far)…
```

The final summary is printed to stdout:

```
Scanned blocks 4609000–4620000: 47 inscriptions found.
Content written to: ./my-inscriptions
```
