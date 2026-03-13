Moderation
==========

`dog` includes a block explorer, which you can run locally with `dog server`.

The block explorer allows viewing inscriptions. Inscriptions are user-generated
content, which may be objectionable or unlawful.

It is the responsibility of each individual who runs an ordinal block explorer
instance to understand their responsibilities with respect to unlawful content,
and decide what moderation policy is appropriate for their instance.

In order to prevent particular inscriptions from being displayed on an `dog`
instance, they can be included in a YAML config file, which is loaded with the
`--config` option.

To hide inscriptions, first create a config file, with the inscription ID you
want to hide:

```yaml
hidden:
- 0000000000000000000000000000000000000000000000000000000000000000i0
```

The suggested name for `dog` config files is `dog.yaml`, but any filename can
be used.

Then pass the file to `--config` when starting the server:

`dog --config dog.yaml server`

Note that the `--config` option comes after `dog` but before the `server`
subcommand.

`dog` must be restarted in order to load changes to the config file.

`doginals.com`
--------------

The `doginals.com` instances use `systemd` to run the `dog server` service,
which is called `dog`, with a config file located at `/var/lib/dog/dog.yaml`.

To hide an inscription on `doginals.com`:

1. SSH into the server
2. Add the inscription ID to `/var/lib/dog/dog.yaml`
3. Restart the service with `systemctl restart dog`
4. Monitor the restart with `journalctl -u dog`

Currently, `dog` is slow to restart, so the site will not come back online
immediately.

