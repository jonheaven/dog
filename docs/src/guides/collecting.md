Collecting
==========

Currently, [dog](https://github.com/jonheaven/dog/) is the only wallet supporting
koinu-control and koinu-selection, which are required to safely store and send
rare koinus and inscriptions, hereafter doginals.

The recommended way to send, receive, and store doginals is with `dog`, but if
you are careful, it is possible to safely store, and in some cases send,
doginals with other wallets.

As a general note, receiving doginals in an unsupported wallet is not
dangerous. Doginals can be sent to any dogecoin address, and are safe as long as
the UTXO that contains them is not spent. However, if that wallet is then used
to send dogecoin, it may select the UTXO containing the ordinal as an input, and
send the inscription or spend it to fees.

A [guide](./collecting/sparrow-wallet.md) to creating a `dog`-compatible wallet with [Sparrow Wallet](https://sparrowwallet.com/) is available
in this handbook.

Please note that if you follow this guide, you should not use the wallet you
create to send DOGE, unless you perform manual coin-selection to avoid sending
doginals.

