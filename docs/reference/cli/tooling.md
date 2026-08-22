# Tooling

Orcker installs developer tools - **Composer**, **Node.js** (`node`/`npm`/`npx`),
**Bun** (`bun`/`bunx`), the **Laravel installer** (`laravel`), and **WP-CLI**
(`wp`) - as self-contained binaries on your `PATH`. Each is identified by a
short `id`: `composer`, `node`, `bun`, `laravel`, or `wp-cli`. The
[Tooling guide](../../guide/tooling) covers the model in depth; this page is the
command reference.

::: info Latest only
Orcker installs the latest stable release of each tool (latest **LTS** for Node).
There is no per-version selection - installing again updates to the current
latest. Installing your first tool from the CLI **automatically adds** Orcker's bin
directory to your `PATH`; you can also manage it yourself with
[`orcker path install`](#path-setup). If the bin directory isn't on your `PATH`,
[`orcker doctor`](./diagnostics) flags it with the one-line fix.
:::

## Listing

| Command | Description |
| --- | --- |
| `orcker tools` | List every tool: install status, installed version, and the commands it provides. |

```sh
orcker tools
```

```text
TOOL      STATUS          COMMANDS       LOCATION
composer  2.10.1          composer       -
node      external        node,npm,npx   /opt/homebrew/bin/node
bun       not installed   bun,bunx       -
```

`LOCATION` is only populated for `external` tools - ones already on your
`PATH` from somewhere other than Orcker (Homebrew, `nvm`/`fnm`, a global
Composer, …). See the [Tooling guide](../../guide/tooling#external-tools) for
what that means and why there's no install/update action for them.

Add `--json` for machine-readable output.

## Installing & updating

| Command | Description | Example |
| --- | --- | --- |
| `orcker install tool <ID>` | Install the tool's latest version, then expose its commands on `PATH` - a **verified release download** for `node` / `bun` / `composer`, or a **Composer build** (`create-project`) for `laravel` / `wp-cli`. **Idempotent** - run again to update to the current latest. | `orcker install tool node` |
| `orcker uninstall tool <ID>` | Remove the tool's files and its `PATH` commands. | `orcker uninstall tool bun` |

```sh
orcker install tool composer    # PHP dependency manager (needs a PHP version)
orcker install tool node        # latest Node LTS - node, npm, npx
orcker install tool bun         # bun + bunx
orcker install tool laravel     # the laravel new installer (needs Composer)
orcker install tool wp-cli      # the wp command for WordPress (needs Composer)
orcker install tool node        # run again to update to the newest LTS
orcker uninstall tool bun       # remove bun and prune its shims
```

`<ID>` is one of `composer`, `node`, `bun`, `laravel`, or `wp-cli`. An unknown
id returns a `not_found` error.

::: warning Composer requires PHP
`composer` runs under Orcker's managed PHP, so install at least one
[PHP version](./php) first. Node and Bun are standalone. The Laravel installer
and WP-CLI are Composer packages, so they also need Orcker's own Composer
installed first.
:::

::: tip WP-CLI has no phar self-update
Orcker's `wp-cli` is a Composer install, so WP-CLI's own `wp cli update`
subcommand isn't applicable and will error - run `orcker install tool wp-cli`
again instead to update.
:::

## PATH setup

The tool commands live in Orcker's `{data}/bin` directory. Manage your shell's
`PATH` entry for it with `orcker path`:

| Command | Description |
| --- | --- |
| `orcker path install` | Add `{data}/bin` to your shell startup file (idempotent; covers zsh, bash, and fish). |
| `orcker path uninstall` | Remove the Orcker `PATH` block from your shell startup file. |
| `orcker path print` | Print the shell snippet without modifying any file (for `eval` / manual use). |

```sh
orcker path install     # then open a new terminal
```

## Exit codes

These commands follow the standard CLI [exit codes](./#exit-codes): `0` on
success, `1` on a daemon error (e.g. an unknown tool id, a failed download, or a
checksum mismatch), and `69` if the daemon is unreachable.

## See also

- [Tooling guide](../../guide/tooling) - the full model and where files live.
- [PHP reference](./php) - the version model these tools follow.
- [Services reference](./services) - the same install-on-demand approach.
