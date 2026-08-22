# Services

Orcker installs and supervises local database, cache, and search engines as native,
per-user processes - no Docker. Each service is identified by a short `id`:
`redis`, `mysql`, `mariadb`, `postgres`, `meilisearch`, or - per site -
`reverb:<site>` (see [Instances](#instances)). The [Services & Databases
guide](../../guide/services) covers the model in depth; this page is the command
reference. For creating and managing the databases *inside* a SQL engine, see
[Databases](./db).

::: info Redis is Valkey
The `redis` slot is served by **Valkey** (the BSD-licensed, wire-compatible fork).
It is displayed as `Redis (Valkey)` and your clients are unaffected.
:::

## Listing

| Command | Description |
| --- | --- |
| `orcker services` | List every known service: installed version, run state (running / stopped / failed), port, and whether it hosts databases. |
| `orcker service available` | List the versions installable from Orcker's hosted distribution for your platform, tagging any already installed. |

```sh
orcker services             # what's installed and running
orcker service available    # what you could install
```

## Installing & versioning

| Command | Description | Example |
| --- | --- | --- |
| `orcker service install <SVC> <VERSION>` | Download and install a service build, then start and enable it. | `orcker service install redis 8` |
| `orcker service change-version <SVC> <VERSION>` | Switch an installed service to a different version (the data directory is kept). | `orcker service change-version postgres 16.2` |
| `orcker service uninstall <SVC> <VERSION> [--purge]` | Remove an installed version. Add `--purge` to also delete the engine's stored data (destructive). | `orcker service uninstall mysql 8.4 --purge` |

```sh
orcker service install redis 8           # install + start + enable
orcker service change-version redis 8.1  # upgrade in place, keep data
orcker service uninstall redis 8         # remove binaries, keep data
orcker service uninstall redis 8 --purge # remove binaries AND data
```

::: warning `--purge` deletes data
Without `--purge`, uninstalling keeps the data directory so a later reinstall
picks up where you left off. With `--purge` the engine's stored data is deleted -
there is no undo.
:::

::: info PostgreSQL has a `full` (PostGIS) variant
`postgres` publishes two builds per major: the lean base (`17`) and a PostGIS
build (`17-full`). Install either by its label, e.g.
`orcker service install postgres 17-full`. The two are separate installs that
**share one data directory** (pinned to the numeric major), so `change-version`
between them preserves your databases; see
[PostgreSQL: base and PostGIS builds](../../guide/services#postgresql-base-and-postgis-full-builds)
for the extension lists, the shared-datadir behaviour, and the GPL posture of
`full`.
:::

## Lifecycle

| Command | Description |
| --- | --- |
| `orcker service start <SVC>` | Start the service now. |
| `orcker service stop <SVC>` | Stop the service for the current session. Installed engines auto-start again on the next daemon start; `uninstall` to keep one off. |
| `orcker service restart <SVC>` | Restart the running service. |

```sh
orcker service start postgres
orcker service stop postgres
orcker service restart postgres
```

## Configuration

| Command | Description | Example |
| --- | --- | --- |
| `orcker service set-port <SVC> <PORT>` | Set the loopback port the service listens on. Applies on the next start/restart. | `orcker service set-port redis 6380` |
| `orcker service set <SVC> <KEY> <VALUE>` | Set a free-form config directive for the engine. Applies on the next start/restart. | `orcker service set mysql max_allowed_packet 256M` |
| `orcker service unset <SVC> <KEY>` | Remove a directive Orcker is overriding, so the engine's own default applies again. | `orcker service unset mysql max_allowed_packet` |
| `orcker service overrides <SVC>` | List the directives currently set for a service (`no overrides` when there are none). | `orcker service overrides mysql` |
| `orcker service logs <SVC> [--lines <N>]` | Print the tail of the service's log. `--lines` defaults to 100. | `orcker service logs mysql --lines 200` |

```sh
orcker service set-port redis 6380
orcker service logs mysql              # last 100 lines
orcker service logs mysql --lines 50
```

Default ports: Redis `6379`, MySQL / MariaDB `3306` (they share the port, so only
one can be enabled on it at a time), PostgreSQL `5432`, Meilisearch `7700`.

### Configuration overrides

`set` / `unset` / `overrides` manage free-form directives for the engine's *own*
config file - the way `orcker php ini` does for a PHP version. Orcker renders them
into a sidecar the engine reads after Orcker's own settings, so an override wins:

```sh
orcker service set mysql max_allowed_packet 256M
orcker service set mysql sql_mode STRICT_TRANS_TABLES,NO_ZERO_DATE
orcker service overrides mysql
#   max_allowed_packet = 256M
#   sql_mode = STRICT_TRANS_TABLES,NO_ZERO_DATE
orcker service unset mysql sql_mode
orcker service restart mysql           # overrides apply on the next start
```

Supported by the config-backed engines only: `mysql`, `mariadb`, `postgres`, and
`redis`. Meilisearch and Reverb are argv/env driven, so they answer
`does not support configuration overrides`.

Names and values are **shape-validated** client-side before connecting (and again
by the daemon), but not semantically: whether the engine accepts a directive is
the engine's business, and a bad one surfaces when the service next starts.
Directives Orcker manages through typed paths are refused with a pointer to the
right command - the port (use `orcker service set-port`), the data directory, the
socket, the pid file, logging (read it with `orcker service logs`), the
MySQL/MariaDB bootstrap `init-file`, the loopback binding, and the engines' own
`include` directives. The check folds case in every dialect, and `-`/`_` for
MySQL/MariaDB, so `Bind_Address` is refused just as `bind-address` is.

::: warning Restart to apply
Like `set-port`, setting an override never restarts anything. Run
`orcker service restart <SVC>` when you're ready for it to take effect. If the
engine then refuses to start, the error carries the tail of its own log plus the
path to the hand-edit file - see the
[Services & Databases guide](../../guide/services#getting-a-directive-wrong).
:::

Hand edits that Orcker must never touch go in the service's `conf.d/50-local.<ext>`
file instead, which is created once and never rewritten. `orcker doctor` scans it
and warns about reserved or malformed lines. See
[Service configuration overrides](../../guide/services#service-configuration-overrides)
for the two-file model, and the [Configuration
Reference](../configuration#services-id) for how overrides are stored.

## Instances

The commands above address a service by its **wire id**. For an engine that only
ever has one instance, the id is just the type (`mysql`, `redis`). Per-site types
- Laravel Reverb today - can have one instance per site, and their ids carry the
site: `reverb:blog`.

| Command | Description | Example |
| --- | --- | --- |
| `orcker service add --type <TYPE> [--site <SITE>] [--port <PORT>] [--version <VERSION>] [--autostart on\|off]` | Add a new instance of a service type. | `orcker service add --type reverb --site blog` |
| `orcker service remove <SVC> [--purge]` | Remove a per-site instance. Add `--purge` to delete its stored state too. | `orcker service remove reverb:blog` |
| `orcker service set-autostart <SVC> on\|off` | Set whether the instance starts with Orcker. | `orcker service set-autostart redis off` |
| `orcker service set-site <SVC> <SITE>` | Re-link a per-site instance to a different site. | `orcker service set-site reverb:blog shop` |

```sh
orcker service add --type reverb --site blog     # reverb:blog, on the next free port
orcker service add --type postgres --version 17  # an engine instance, explicit version
orcker service set-autostart reverb:blog on
orcker service set-site reverb:blog shop         # becomes reverb:shop
orcker service remove reverb:blog --purge
```

- `--type` is a type id (`redis`, `mysql`, `mariadb`, `postgres`, `meilisearch`,
  `reverb`), not a wire id.
- `--site` is **required** for a per-site type, and must name a **linked Laravel**
  site (one with an `artisan` file). The instance runs against that site's PHP and
  document root, which is why `--version` doesn't apply to it.
- `--version` is required for a versioned type - `add` installs that version as
  part of the call.
- `--port` defaults to the next free loopback port at or above the type's default.
  An explicit port already reserved by another instance is refused.
- `--autostart` defaults per type: engines start with Orcker, per-site app servers
  do not.
- `remove` is for per-site instances. Removing an engine's *installed version* is
  [`orcker service uninstall`](#installing-versioning).

## See also

- [Services & Databases guide](../../guide/services) - the supervision model and posture
- [Databases](./db) - creating, dropping, backing up databases inside a SQL engine
- [Configuration Reference](../configuration) - the `[services.<id>]` config tables
- [orcker-services](../../developer/crates/orcker-services) - the crate behind these commands
