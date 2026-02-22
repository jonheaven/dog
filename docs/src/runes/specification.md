Runes Does Not Have a Specification
===================================

The Runes reference implementation, `ord`, is the normative specification of
the Runes protocol.

Nothing you read here or elsewhere, aside from the code of `ord`, is a
specification. This prose description of the dunes protocol is provided as a
guide to the behavior of `ord`, and the code of `ord` itself should always be
consulted to confirm the correctness of any prose description.

If, due to a bug in `ord`, this document diverges from the actual behavior of
`ord` and it is impractically disruptive to change `ord`'s behavior, this
document will be amended to agree with `ord`'s actual behavior.

Users of alternative implementations do so at their own risk, and services
wishing to integrate Runes are strongly encouraged to use `ord` itself to make
Runes transactions, and to determine the state of dunes, mints, and balances.

Dunestones
----------

Dune protocol messages are termed "dunestones".

The Runes protocol activates on block 840,000. Dunestones in earlier blocks are
ignored.

Abstractly, dunestones contain the following fields:

```rust
struct Dunestone {
  edicts: Vec<Edict>,
  etching: Option<Etching>,
  mint: Option<DuneId>,
  pointer: Option<u32>,
}
```

Runes are created by etchings:

```rust
struct Etching {
  divisibility: Option<u8>,
  premine: Option<u128>,
  dune: Option<Dune>,
  spacers: Option<u32>,
  symbol: Option<char>,
  terms: Option<Terms>,
}
```

Which may contain mint terms:

```rust
struct Terms {
  amount: Option<u128>,
  cap: Option<u128>,
  height: (Option<u64>, Option<u64>),
  offset: (Option<u64>, Option<u64>),
}
```

Runes are transferred by edict:

```rust
struct Edict {
  id: DuneId,
  amount: u128,
  output: u32,
}
```

Dune IDs are encoded as the block height and transaction index of the
transaction in which the dune was etched:

```rust
struct DuneId {
  block: u64,
  tx: u32,
}
```

Dune IDs are represented in text as `BLOCK:TX`.

Dune names are encoded as modified base-26 integers:

```rust
struct Dune(u128);
```

### Deciphering

Dunestones are deciphered from transactions with the following steps:

1. Find the first transaction output whose script pubkey begins with `OP_RETURN
   OP_13`.

2. Concatenate all following data pushes into a payload buffer.

3. Decode a sequence 128-bit [LEB128](https://en.wikipedia.org/wiki/LEB128)
   integers from the payload buffer.

4. Parse the sequence of integers into an untyped message.

5. Parse the untyped message into a dunestone.

Deciphering may produce a malformed dunestone, termed a
[cenotaph](https://en.wikipedia.org/wiki/Cenotaph).

#### Locating the Dunestone Output

Outputs are searched for the first script pubkey that beings with `OP_RETURN
OP_13`. If deciphering fails, later matching outputs are not considered.

#### Assembling the Payload Buffer

The payload buffer is assembled by concatenating data pushes, after `OP_13`, in
the matching script pubkey.

Data pushes are opcodes 0 through 78 inclusive. If a non-data push opcode is
encountered, i.e., any opcode equal to or greater than opcode 79, the
deciphered dunestone is a cenotaph with no etching, mint, or edicts.

#### Decoding the Integer Sequence

A sequence of 128-bit integers are decoded from the payload as LEB128 varints.

LEB128 varints are encoded as sequence of bytes, each of which has the
most-significant bit set, except for the last.

If a LEB128 varint contains more than 18 bytes, would overflow a u128, or is
truncated, meaning that the end of the payload buffer is reached before
encountering a byte with the continuation bit not set, the decoded dunestone is
a cenotaph with no etching, mint, or edicts.

#### Parsing the Message

The integer sequence is parsed into an untyped message:

```rust
struct Message {
  fields: Map<u128, Vec<u128>>,
  edicts: Vec<Edict>,
}
```

The integers are interpreted as a sequence of tag/value pairs, with duplicate
tags appending their value to the field value.

If a tag with value zero is encountered, all following integers are interpreted
as a series of four-integer edicts, each consisting of a dune ID block height,
dune ID transaction index, amount, and output.

```rust
struct Edict {
  id: DuneId,
  amount: u128,
  output: u32,
}
```

Dune ID block heights and transaction indices in edicts are delta encoded.

Edict dune ID decoding starts with a base block height and transaction index of
zero. When decoding each dune ID, first the encoded block height delta is added
to the base block height. If the block height delta is zero, the next integer
is a transaction index delta. If the block height delta is greater than zero,
the next integer is instead an absolute transaction index.

This implies that edicts must first be sorted by dune ID before being encoded
in a dunestone.

For example, to encode the following edicts:

| block | TX | amount | output |
|-------|----|--------|--------|
| 10    | 5  | 5      | 1      |
| 50    | 1  | 25     | 4      |
| 10    | 7  | 1      | 8      |
| 10    | 5  | 10     | 3      |

They are first sorted by block height and transaction index:

| block | TX | amount | output |
|-------|----|--------|--------|
| 10    | 5  | 5      | 1      |
| 10    | 5  | 10     | 3      |
| 10    | 7  | 1      | 8      |
| 50    | 1  | 25     | 4      |

And then delta encoded as:

| block delta | TX delta | amount | output |
|-------------|----------|--------|--------|
| 10          | 5        | 5      | 1      |
| 0           | 0        | 10     | 3      |
| 0           | 2        | 1      | 8      |
| 40          | 1        | 25     | 4      |

If an edict output is greater than the number of outputs of the transaction, an
edict dune ID is encountered with block zero and nonzero transaction index, or
a field is truncated, meaning a tag is encountered without a value, the decoded
dunestone is a cenotaph.

Note that if a cenotaph is produced here, the cenotaph is not empty, meaning
that it contains the fields and edicts, which may include an etching and mint.

#### Parsing the Dunestone

The dunestone:

```rust
struct Dunestone {
  edicts: Vec<Edict>,
  etching: Option<Etching>,
  mint: Option<DuneId>,
  pointer: Option<u32>,
}
```

Is parsed from the unsigned message using the following tags:

```rust
enum Tag {
  Body = 0,
  Flags = 2,
  Dune = 4,
  Premine = 6,
  Cap = 8,
  Amount = 10,
  HeightStart = 12,
  HeightEnd = 14,
  OffsetStart = 16,
  OffsetEnd = 18,
  Mint = 20,
  Pointer = 22,
  Cenotaph = 126,

  Divisibility = 1,
  Spacers = 3,
  Symbol = 5,
  Nop = 127,
}
```

Note that tags are grouped by parity, i.e., whether they are even or odd.
Unrecognized odd tags are ignored. Unrecognized even tags produce a cenotaph.

All unused tags are reserved for use by the protocol, may be assigned at any
time, and should not be used.

##### Body

The `Body` tag marks the end of the dunestone's fields, causing all following
integers to be interpreted as edicts.

##### Flags

The `Flag` field contains a bitmap of flags, whose position is `1 <<
FLAG_VALUE`:

```rust
enum Flag {
  Etching = 0,
  Terms = 1,
  Turbo = 2,
  Cenotaph = 127,
}
```

The `Etching` flag marks this transaction as containing an etching.

The `Terms` flag marks this transaction's etching as having open mint terms.

The `Turbo` flag marks this transaction's etching as opting into future
protocol changes. These protocol changes may increase light client validation
costs, or just be highly degenerate.

The `Cenotaph` flag is unrecognized.

If the value of the flags field after removing recognized flags is nonzero, the
dunestone is a cenotaph.

##### Dune

The `Dune` field contains the name of the dune being etched. If the `Etching`
flag is set but the `Dune` field is omitted, a reserved dune name is
allocated.

##### Premine

The `Premine` field contains the amount of premined dunes.

##### Cap

The `Cap` field contains the allowed number of mints.

##### Amount

The `Amount` field contains the amount of dunes each mint transaction receives.

##### HeightStart and HeightEnd

The `HeightStart` and `HeightEnd` fields contain the mint's starting and ending
absolute block heights, respectively. The mint is open starting in the block
with height `HeightStart`, and closes in the block with height `HeightEnd`.

##### OffsetStart and OffsetEnd

The `OffsetStart` and `OffsetEnd` fields contain the mint's starting and ending
block heights, relative to the block in which the etching is mined. The mint is
open starting in the block with height `OffsetStart` + `ETCHING_HEIGHT`, and
closes in the block with height `OffsetEnd` + `ETCHING_HEIGHT`.

##### Mint

The `Mint` field contains the Dune ID of the dune to be minted in this
transaction.

##### Pointer

The `Pointer` field contains the index of the output to which dunes unallocated
by edicts should be transferred. If the `Pointer` field is absent, unallocated
dunes are transferred to the first non-`OP_RETURN` output. If the pointer is
greater than or equal to the number of outputs, the dunestone is a cenotaph.

##### Cenotaph

The `Cenotaph` field is unrecognized.

##### Divisibility

The `Divisibility` field, raised to the power of ten, is the number of subunits
in a super unit of dunes.

For example, the amount `1234` of different dunes with divisibility 0 through 3
is displayed as follows:

| Divisibility | Display |
|--------------|---------|
| 0            | 1234    |
| 1            | 123.4   |
| 2            | 12.34   |
| 3            | 1.234   |

##### Spacers

The `Spacers` field is a bitfield of `•` spacers that should be displayed
between the letters of the dune's name.

The Nth field of the bitfield, starting from the least significant, determines
whether or not a spacer should be displayed between the Nth and N+1th
character, starting from the left of the dune's name.

For example, the dune name `AAAA` rendered with different spacers:

| Spacers | Display |
|---------|---------|
| 0b1     | A•AAA   |
| 0b11    | A•A•AA  |
| 0b10    | AA•AA   |
| 0b111   | A•A•A•A |

Trailing spacers are ignored.

##### Symbol

The `Symbol` field is the Unicode codepoint of the Dune's currency symbol,
which should be displayed after amounts of that dune. If a dune does not have a
currency symbol, the generic currency character `¤` should be used.

For example, if the `Symbol` is `#` and the divisibility is 2, the amount of
`1234` units should be displayed as `12.34 #`.

##### Nop

The `Nop` field is unrecognized.

#### Cenotaphs

Cenotaphs have the following effects:

- All dunes input to a transaction containing a cenotaph are burned.

- If the dunestone that produced the cenotaph contained an etching, the etched
  dune has supply zero and is unmintable.

- If the dunestone that produced the cenotaph is a mint, the mint counts
  against the mint cap and the minted dunes are burned.

Cenotaphs may be created if a dunestone contains an unrecognized even tag, an
unrecognized flag, an edict with an output number greater than the number of
outputs, a dune ID with block zero and nonzero transaction index, a malformed
varint, a non-datapush instruction in the dunestone output script pubkey, a tag
without a following value, trailing integers not part of an edict, or a pointer
greater than or equal to the number of outputs.

#### Executing the Dunestone

Dunestones are executed in the order their transactions are included in blocks.

##### Etchings

A dunestone may contain an etching:

```rust
struct Etching {
  divisibility: Option<u8>,
  premine: Option<u128>,
  dune: Option<Dune>,
  spacers: Option<u32>,
  symbol: Option<char>,
  terms: Option<Terms>,
}
```

`dune` is the name of the dune to be etched, encoded as modified base-26
integer.

Dune names consist of the letters A through Z, with the following encoding:

| Name | Encoding |
|------|----------|
| A    | 0        |
| B    | 1        |
| …    | …        |
| Y    | 24       |
| Z    | 25       |
| AA   | 26       |
| AB   | 27       |
| …    | …        |
| AY   | 50       |
| AZ   | 51       |
| BA   | 52       |

And so on and so on.

Dune names `AAAAAAAAAAAAAAAAAAAAAAAAAAA` and above are reserved.

If `dune` is omitted a reserved dune name is allocated as follows:

```rust
fn reserve(block: u64, tx: u32) -> Dune {
  Dune(
    6402364363415443603228541259936211926
    + (u128::from(block) << 32 | u128::from(tx))
  )
}
```

`6402364363415443603228541259936211926` corresponds to the dune name
`AAAAAAAAAAAAAAAAAAAAAAAAAAA`.

If `dune` is present, it must be unlocked as of the block in which the etching
appears.

Initially, all dune names of length thirteen and longer, up until the first
reserved dune name, are unlocked.

Runes begin unlocking in block 840,000, the block in which the dunes protocol
activates.

Thereafter, every 17,500 block period, the next shortest length of dune names
is continuously unlocked. So, between block 840,000 and block 857,500, the
twelve-character dune names are unlocked, between block 857,500 and block
875,000 the eleven character dune names are unlocked, and so on and so on,
until the one-character dune names are unlocked between block 1,032,500 and
block 1,050,000. See the `ord` codebase for the precise unlocking schedule.

To prevent front running an etching that has been broadcast but not mined, if a
non-reserved dune name is being etched, the etching transaction must contain a
valid commitment to the name being etched.

A commitment consists of a data push of the dune name, encoded as a
little-endian integer with trailing zero bytes elided, present in an input
witness tapscript where the output being spent has at least six confirmations.

If a valid commitment is not present, the etching is ignored.

#### Minting

A dunestone may mint a dune by including the dune's ID in the `Mint` field.

If the mint is open, the mint amount is added to the unallocated dunes in the
transaction's inputs. These dunes may be transferred using edicts, and will
otherwise be transferred to the first non-`OP_RETURN` output, or the output
designated by the `Pointer` field.

Mints may be made in any transaction after an etching, including in the same
block.

#### Transferring

Runes are transferred by edict:

```rust
struct Edict {
  id: DuneId,
  amount: u128,
  output: u32,
}
```

A dunestone may contain any number of edicts, which are processed in sequence.

Before edicts are processed, input dunes, as well as minted or premined dunes,
if any, are unallocated.

Each edict decrements the unallocated balance of dune `id` and increments the
balance allocated to transaction outputs of dune `id`.

If an edict would allocate more dunes than are currently unallocated, the
`amount` is reduced to the number of currently unallocated dunes. In other
words, the edict allocates all remaining unallocated units of dune `id`.

Because the ID of an etched dune is not known before it is included in a block,
ID `0:0` is used to mean the dune being etched in this transaction, if any.

An edict with `amount` zero allocates all remaining units of dune `id`.

An edict with `output` equal to the number of transaction outputs allocates
`amount` dunes to each non-`OP_RETURN` output in order.

An edict with `amount` zero and `output` equal to the number of transaction
outputs divides all unallocated units of dune `id` between each non-`OP_RETURN`
output. If the number of unallocated dunes is not divisible by the number of
non-`OP_RETURN` outputs, 1 additional dune is assigned to the first `R`
non-`OP_RETURN` outputs, where `R` is the remainder after dividing the balance
of unallocated units of dune `id` by the number of non-`OP_RETURN` outputs.

If any edict in a dunestone has a dune ID with `block` zero and `tx` greater
than zero, or `output` greater than the number of transaction outputs, the
dunestone is a cenotaph.

Note that edicts in cenotaphs are not processed, and all input dunes are
burned.
