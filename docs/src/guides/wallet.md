Wallet
======

Individual sats can be inscribed with arbitrary content, creating
Dogecoin-native digital artifacts that can be held in a Dogecoin wallet and
transferred using Dogecoin transactions. Inscriptions are as durable, immutable,
secure, and decentralized as Dogecoin itself.

Working with inscriptions requires a Dogecoin full node, to give you a view of
the current state of the Dogecoin blockchain, and a wallet that can create
inscriptions and perform sat control when constructing transactions to send
inscriptions to another wallet.

Dogecoin Core provides both a Dogecoin full node and wallet. However, the Dogecoin
Core wallet cannot create inscriptions and does not perform sat control.

This requires [`dog`](https://github.com/jonheaven/dog), the Doginal utility. `dog`
doesn't implement its own wallet, so `dog wallet` subcommands interact with
Dogecoin Core wallets.

This guide covers:

1. Installing Dogecoin Core
2. Syncing the Dogecoin blockchain
3. Creating a Dogecoin Core wallet
4. Using `dog wallet receive` to receive sats
5. Creating inscriptions with `dog wallet inscribe`
6. Sending inscriptions with `dog wallet send`
7. Receiving inscriptions with `dog wallet receive`
8. Batch inscribing with `dog wallet inscribe --batch`

Getting Help
------------

If you get stuck, try asking for help on the [Doginals Discord
Server](https://discord.com/invite/87cjuz4FYg), or checking GitHub for relevant
[issues](https://github.com/jonheaven/dog/issues) and
[discussions](https://github.com/jonheaven/dog/discussions).

Installing Dogecoin Core
-----------------------

Dogecoin Core is available from [dogecoin.com](https://dogecoin.com/wallets).

Making inscriptions requires Dogecoin Core 28 or newer.

This guide does not cover installing Dogecoin Core in detail. Once Dogecoin Core
is installed, you should be able to run `dogecoind -version` successfully from
the command line. Do *NOT* use `dogecoin-qt`.

Configuring Dogecoin Core
------------------------

`dog` requires Dogecoin Core's transaction index and rest interface.

To configure your Dogecoin Core node to maintain a transaction
index, add the following to your `dogecoin.conf`:

```
txindex=1
```

Or, run `dogecoind` with `-txindex`:

```
dogecoind -txindex
```

Details on creating or modifying your `dogecoin.conf` file can be found in the
upstream reference documentation
[here](https://github.com/bitcoin/bitcoin/blob/master/doc/bitcoin-conf.md).

Syncing the Dogecoin Blockchain
------------------------------

To sync the chain, run:

```
dogecoind -txindex
```

…and leave it running until `getblockcount`:

```
dogecoin-cli getblockcount
```

agrees with the block count on a block explorer like [the mempool.space block
explorer](https://mempool.space/). `dog` interacts with `dogecoind`, so you
should leave `dogecoind` running in the background when you're using `dog`.

The blockchain takes about 600GB of disk space. If you have an external drive
you want to store blocks on, use the configuration option
`blocksdir=<external_drive_path>`. This is much simpler than using the
`datadir` option because the cookie file will still be in the default location
for `dogecoin-cli` and `dog` to find.

Troubleshooting
---------------

Make sure you can access `dogecoind` with `dogecoin-cli -getinfo` and that it is
fully synced.

If `dogecoin-cli -getinfo` returns `Could not connect to the server`, `dogecoind`
is not running.

Make sure `rpcuser`, `rpcpassword`, or `rpcauth` are *NOT* set in your
`dogecoin.conf` file. `dog` requires using cookie authentication. Make sure there
is a file `.cookie` in your dogecoin data directory.

If `dogecoin-cli -getinfo` returns `Could not locate RPC credentials`, then you
must specify the cookie file location.
If you are using a custom data directory (specifying the `datadir` option),
then you must specify the cookie location like
`dogecoin-cli -rpccookiefile=<your_dogecoin_datadir>/.cookie -getinfo`.
When running `dog` you must specify the cookie file location with
`--cookie-file=<your_dogecoin_datadir>/.cookie`.

Make sure you do *NOT* have `disablewallet=1` in your `dogecoin.conf` file. If
`dogecoin-cli listwallets` returns `Method not found` then the wallet is disabled
and you won't be able to use `dog`.

Make sure `txindex=1` is set. Run `dogecoin-cli getindexinfo` and it should
return something like
```json
{
  "txindex": {
    "synced": true,
    "best_block_height": 776546
  }
}
```
If it only returns `{}`, `txindex` is not set.
If it returns `"synced": false`, `dogecoind` is still creating the `txindex`.
Wait until `"synced": true` before using `dog`.

If you have `maxuploadtarget` set it can interfere with fetching blocks for
`dog` index. Either remove it or set `whitebind=127.0.0.1:8333`.

Installing `dog`
----------------

The `dog` utility is written in Rust and can be built from
[source](https://github.com/jonheaven/dog). Pre-built binaries are available on the
[releases page](https://github.com/jonheaven/dog/releases).

You can install the latest pre-built binary from the command line with:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://doginals.com/install.sh | bash -s
```

Once `dog` is installed, you should be able to run:

```
dog --version
```

Which prints out `dog`'s version number.

Creating a Wallet
-----------------

`dog` uses `dogecoind` to manage private keys, sign transactions, and
broadcast transactions to the Dogecoin network. Additionally the `dog wallet`
requires [`dog server`](explorer.md) running in the background. Make sure these
programs are running:

```
dogecoind -txindex
```

```
dog server
```

To create a wallet named `dog`, the default, for use with `dog wallet`, run:

```
dog wallet create
```

This will print out your seed phrase mnemonic, store it somewhere safe.

```
{
  "mnemonic": "dignity buddy actor toast talk crisp city annual tourist orient similar federal",
  "passphrase": ""
}
```

If you want to specify a different name or use an `dog server` running on a
non-default URL you can set these options:

```
dog wallet --name foo --server-url http://127.0.0.1:8080 create
```

To see all available wallet options you can run:

```
dog wallet help
```

Restoring and Dumping Wallet
----------------------------

The `dog` wallet uses descriptors, so you can export the output descriptors and
import them into another descriptor-based wallet. To export the wallet
descriptors, which include your private keys:

```
$ dog wallet dump
==========================================
= THIS STRING CONTAINS YOUR PRIVATE KEYS =
=        DO NOT SHARE WITH ANYONE        =
==========================================
{
  "wallet_name": "dog",
  "descriptors": [
    {
      "desc": "tr([551ac972/86'/1'/0']tprv8h4xBhrfZwX9o1XtUMmz92yNiGRYjF9B1vkvQ858aN1UQcACZNqN9nFzj3vrYPa4jdPMfw4ooMuNBfR4gcYm7LmhKZNTaF4etbN29Tj7UcH/0/*)#uxn94yt5",
      "timestamp": 1296688602,
      "active": true,
      "internal": false,
      "range": [
        0,
        999
      ],
      "next": 0
    },
    {
      "desc": "tr([551ac972/86'/1'/0']tprv8h4xBhrfZwX9o1XtUMmz92yNiGRYjF9B1vkvQ858aN1UQcACZNqN9nFzj3vrYPa4jdPMfw4ooMuNBfR4gcYm7LmhKZNTaF4etbN29Tj7UcH/1/*)#djkyg3mv",
      "timestamp": 1296688602,
      "active": true,
      "internal": true,
      "range": [
        0,
        999
      ],
      "next": 0
    }
  ]
}
```

An `dog` wallet can be restored from a mnemonic:

```
dog wallet restore --from mnemonic
```

Type your mnemonic and press return.

To restore from a descriptor in `descriptor.json`:

```
cat descriptor.json | dog wallet restore --from descriptor
```

To restore from a descriptor in the clipboard:

```
dog wallet restore --from descriptor
```

Paste the descriptor into the terminal and press CTRL-D on unix and CTRL-Z
on Windows.

Receiving Sats
--------------

Inscriptions are made on individual sats, using normal Dogecoin transactions
that pay fees in sats, so your wallet will need some sats.

Get a new address from your `dog` wallet by running:

```
dog wallet receive
```

And send it some funds.

You can see pending transactions with:

```
dog wallet transactions
```

Once the transaction confirms, you should be able to see the transactions
outputs with `dog wallet outputs`.

Creating Inscription Content
----------------------------

Sats can be inscribed with any kind of content, but the `dog` wallet only
supports content types that can be displayed by the `dog` block explorer.

Additionally, inscriptions are included in transactions, so the larger the
content, the higher the fee that the inscription transaction must pay.

Inscription content is included in transaction witnesses, which receive the
witness discount. To calculate the approximate fee that an inscribe transaction
will pay, divide the content size by four and multiply by the fee rate.

Inscription transactions must be less than 400,000 weight units, or they will
not be relayed by Dogecoin Core. One byte of inscription content costs one
weight unit. Since an inscription transaction includes not just the inscription
content, limit inscription content to less than 400,000 weight units. 390,000
weight units should be safe.

Creating Inscriptions
---------------------

To create an inscription with the contents of `FILE`, run:

```
dog wallet inscribe --fee-rate FEE_RATE --file FILE
```

Ord will output two transactions IDs, one for the commit transaction, and one
for the reveal transaction, and the inscription ID. Inscription IDs are of the
form `TXIDiN`, where `TXID` is the transaction ID of the reveal transaction,
and `N` is the index of the inscription in the reveal transaction.

The commit transaction commits to a tapscript containing the content of the
inscription, and the reveal transaction spends from that tapscript, revealing
the content on chain and inscribing it on the first sat of the input that
contains the corresponding tapscript.

Wait for the reveal transaction to be mined. You can check the status of the
commit and reveal transactions using  [the mempool.space block
explorer](https://mempool.space/).

Once the reveal transaction has been mined, the inscription ID should be
printed when you run:

```
dog wallet inscriptions
```

Parent-Child Inscriptions
-------------------------

Parent-child inscriptions enable what is colloquially known as collections, see
[provenance](../inscriptions/provenance.md) for more information.

To make an inscription a child of another, the parent inscription has to be
inscribed and present in the wallet. To choose a parent run `dog wallet inscriptions`
and copy the inscription id (`<PARENT_INSCRIPTION_ID>`).

Now inscribe the child inscription and specify the parent like so:

```
dog wallet inscribe --fee-rate FEE_RATE --parent <PARENT_INSCRIPTION_ID> --file CHILD_FILE
```

This relationship cannot be added retroactively, the parent has to be
present at inception of the child.

Sending Inscriptions
--------------------

Ask the recipient to generate a new address by running:

```
dog wallet receive
```

Send the inscription by running:

```
dog wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

See the pending transaction with:

```
dog wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt by
running:

```
dog wallet inscriptions
```

Sending Dunes
-------------

Ask the recipient to generate a new address by running:

```
dog wallet receive
```

Send the dunes by running:

```
dog wallet send --fee-rate <FEE_RATE> <ADDRESS> <DUNES_AMOUNT>
```

Where `DUNES_AMOUNT` is the number of dunes to send, a `:` character, and the
name of the dune. For example if you want to send 1000 of the EXAMPLE dune, you
would use `1000:EXAMPLE`.

```
dog wallet send --fee-rate 1 SOME_ADDRESS 1000:EXAMPLE
```

See the pending transaction with:

```
dog wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt with:

```
dog wallet balance
```

Receiving Inscriptions
----------------------

Generate a new receive address using:

```
dog wallet receive
```

The sender can transfer the inscription to your address using:

```
dog wallet send --fee-rate <FEE_RATE> ADDRESS INSCRIPTION_ID
```

See the pending transaction with:
```
dog wallet transactions
```

Once the send transaction confirms, you can confirm receipt by running:

```
dog wallet inscriptions
```

