# Databases

The `orcker db` commands manage the databases *inside* a running SQL service. They
apply to the SQL engines only - `mysql`, `mariadb`, and `postgres`. (Redis is a
cache and has no SQL databases.) To install, start, or stop the engines
themselves, see [Services](./services).

::: tip The engine must be running
Every `orcker db` command connects to a live server, so the target service must be
installed and started first (`orcker service start <svc>`). Database operations need
no elevation - they connect over the local socket / loopback as a passwordless
local-dev superuser.
:::

## Commands

| Command | Description |
| --- | --- |
| `orcker db list <SVC>` | List the databases in the service (system databases are filtered out). |
| `orcker db create <SVC> <NAME>` | Create a new database. |
| `orcker db drop <SVC> <NAME>` | Drop a database (irreversible). |
| `orcker db backup <SVC> <NAME> <FILE>` | Dump a database to a plain-SQL file. |
| `orcker db restore <SVC> <NAME> <FILE>` | Restore a database from a plain-SQL file. |

```sh
orcker db create mysql my_app
orcker db list mysql
orcker db backup mysql my_app ./my_app.sql
orcker db restore mysql my_app ./my_app.sql
orcker db drop mysql my_app
```

## Database names

`create` uses Orcker's portable naming policy. The daemon validates the name before
building SQL. A newly created name must:

- be non-empty and at most **63 characters** (the lowest limit across the engines);
- start with an ASCII letter or underscore (`_`);
- contain only letters, digits, and underscores.

For `drop`, `backup`, and `restore`, the name identifies a database that already
exists. Those operations accept engine-valid names exactly as listed, including
whitespace, punctuation, Unicode, quotes, backticks, periods, and leading hyphens.
Only empty names and names containing NUL are rejected. SQL identifiers are quoted
and escaped per engine, and backup/restore names are passed directly as one process
argument without a shell.

::: warning System databases are protected
Engine-internal databases (`mysql`, `information_schema`, `performance_schema`,
`sys`, `postgres`, `template0`, `template1`) are filtered from `db list` and
cannot be dropped or restored over.
:::

## Backup & restore

`backup` runs the engine's dump tool (`mysqldump`, `mariadb-dump`, or `pg_dump`)
and writes plain SQL. The output is streamed to a temporary file next to the
destination and atomically renamed into place, so a failed dump never truncates an
existing file. A relative `<FILE>` resolves against the current directory.

`restore` replays a plain-SQL file back into an existing database through the
engine's interactive client (`mysql`, `mariadb`, or `psql`) - so the target
database must already exist (`orcker db create` it first if needed).

```sh
# round-trip a database
orcker db create postgres shop
orcker db restore postgres shop ./shop.sql
orcker db backup postgres shop ./shop-$(date +%F).sql
```

## See also

- [Services](./services) - installing and supervising the engines
- [Services & Databases guide](../../guide/services) - the full model
- [orcker-services](../../developer/crates/orcker-services) - `database.rs`, the pure SQL-admin boundary
