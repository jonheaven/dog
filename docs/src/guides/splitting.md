Splitting
=========

Complex transactions can be created using the `dog wallet split` command.

The `split` command takes a YAML configuration file, which specifies any number
of outputs to be created, their dogecoin value, and their value of any number of
dunes. It does not currently allow assigning inscriptions to outputs.

The `split` command can be used to split cardinal, dogecoin-only outputs for
transacting, distribute dunes to large numbers of recipients in a single
transaction.

To send a split transaction using the configuration in `splits.yaml`, run the
following command:

```bash
dog wallet split --fee-rate 21 --splits split.yaml
```

Example `splits.yaml`
--------------------`

```yaml
{{#include ../../../splits.yaml}}
```

