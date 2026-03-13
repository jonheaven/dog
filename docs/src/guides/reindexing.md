Reindexing
==========

Sometimes the `dog` database must be reindexed, which means deleting the
database and restarting the indexing process with either `dog index update` or
`dog server`. Reasons to reindex are:

1. A new major release of dog, which changes the database scheme
2. The database got corrupted somehow

The database `dog` uses is called [redb](https://github.com/cberner/redb),
so we give the index the default file name `index.redb`. By default we store this
file in different locations depending on your operating system.

|Platform | Value                                            | Example                                      |
| ------- | ------------------------------------------------ | -------------------------------------------- |
| Linux   | `$XDG_DATA_HOME`/dog or `$HOME`/.local/share/dog | /home/alice/.local/share/dog                 |
| macOS   | `$HOME`/Library/Application Support/dog          | /Users/Alice/Library/Application Support/dog |
| Windows | `{FOLDERID_RoamingAppData}`\dog                  | C:\Users\Alice\AppData\Roaming\dog           |

So to delete the database and reindex on MacOS you would have to run the following
commands in the terminal:

```bash
rm ~/Library/Application Support/dog/index.redb
dog index update
```

You can of course also set the location of the data directory yourself with `dog
--datadir <DIR> index update` or give it a specific filename and path with `dog
--index <FILENAME> index update`.

