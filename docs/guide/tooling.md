# Tooling

Orcker can install the **developer tools** a typical PHP/Laravel/WordPress project
reaches for - [Composer](https://getcomposer.org), [Node.js](https://nodejs.org)
(with `npm`/`npx`), [Bun](https://bun.sh), the **Laravel installer**, and
**WP-CLI** - the same way it installs [PHP versions](./php-versions):
self-contained binaries fetched on demand (the Laravel installer and WP-CLI are
built via Composer) and dropped onto your `PATH`. No system package manager, no
global install, nothing to uninstall by hand. Already have one installed
elsewhere? Orcker [detects it](#external-tools) and, for most tools, uses it
instead.

| Tool | `id` | Provides | Source |
|---|---|---|---|
| Composer | `composer` | `composer` | getcomposer.org (phar) |
| Node.js | `node` | `node`, `npm`, `npx` | nodejs.org (latest LTS) |
| Bun | `bun` | `bun`, `bunx` | github.com/oven-sh/bun |
| Laravel installer | `laravel` | `laravel` | Composer (`laravel/installer`) |
| WP-CLI | `wp-cli` | `wp` | Composer (`wp-cli/wp-cli-bundle`) |

::: tip Why bundle these?
A fresh machine that has Orcker shouldn't also need Homebrew, `nvm`, or a global
Composer just to run a Laravel app with a Vite front-end. Orcker keeps these tools
in its own data directory, isolated from anything else on your system, and
removes them cleanly on uninstall.
:::

## In the desktop app

Open the **Tooling** page from the sidebar (under the **Developer** group). It
lists the developer tools Orcker manages and their install status:

<ThemedImage light="/images/tooling-light.png" dark="/images/tooling-dark.png" alt="The Tooling page in the Orcker desktop app" />

- **Composer**, **Node**, **Bun**, the **Laravel installer**, and **WP-CLI**,
  each showing the commands it provides.
- Click **Install** to fetch the latest release; once installed you get
  **Update** (re-fetch the current latest) and **Uninstall**.
- A tool you've installed yourself shows an **External** badge and a *Managed by
  you* note. **Install** is still offered, so you can take Orcker's own copy as
  well - see [External tools](#external-tools) below.
- The Laravel installer and WP-CLI are built with Composer, so their **Install**
  button stays disabled until Orcker's own Composer is installed.
- Each tool is placed on your `PATH` alongside PHP and managed entirely by Orcker,
  so it won't collide with a system install.

## From the command line

```sh
orcker tools                      # list the tools and their install status
orcker install tool node          # download + install the latest Node LTS
orcker install tool bun
orcker install tool composer
orcker install tool laravel       # build the Laravel installer (needs Composer)
orcker install tool wp-cli        # build WP-CLI (needs Composer)
orcker uninstall tool bun         # remove a tool and its PATH commands
```

`orcker install tool <id>` is idempotent - run it again to update to the current
latest. See the [Tooling CLI reference](../reference/cli/tooling) for the exact
command surface.

::: tip Updating WP-CLI
Because Orcker's `wp-cli` is a Composer install rather than a phar, WP-CLI's own
`wp cli update` (its phar self-update subcommand) isn't applicable and will
error - use `orcker install tool wp-cli` (or **Update** on the Tooling page)
instead, the same way you wouldn't run `composer self-update` on Orcker's managed
Composer.
:::

## External tools

You don't have to let Orcker manage most of these tools. If you already have
`composer`, `node`, `bun`, or the `laravel` installer available on your `PATH` -
via Homebrew, `nvm`/`fnm`, a global `composer require`, etc. - Orcker **detects**
it and treats it as already available:

- On the **Tooling** page the tool shows an **External** badge (instead of a
  version) and a *Managed by you* note - it's yours to manage, not Orcker's. You
  can still press **Install** to take Orcker's own copy alongside it, which you'll
  need if you want Orcker to build the Laravel installer or WP-CLI.
- The [Laravel site wizard](./sites#create-a-new-laravel-site) and site scaffolding
  accept external Composer / Node / Bun / Laravel as satisfying their
  prerequisites, so you won't be asked to install a second copy. Externally
  installed Composer and the Laravel installer still run under the **Orcker PHP
  version you select**, so versions stay consistent.

A couple of things to know:

- **WP-CLI is the exception: an external `wp` doesn't count.** Orcker runs WP-CLI
  by executing its own build directly rather than by calling `wp` on your `PATH`,
  so it always needs its own copy and never reports one you installed yourself as
  *External*. Your `wp` keeps working exactly as before; the
  [WordPress site wizard](./sites#create-a-new-wordpress-site) will simply offer
  to install Orcker's WP-CLI as a prerequisite.
- **Managed tools win.** If a tool is both Orcker-installed and on your `PATH`, the
  Orcker-managed one takes precedence (its `{data}/bin` shim is earlier on `PATH`).
  That includes `composer`, so installing Orcker's copy changes which `composer`
  your other projects get.
- **Building the *managed* Laravel installer or WP-CLI needs Orcker's own
  Composer.** An external Composer is fine for *scaffolding*, but it can't build
  Orcker's managed `laravel`/`wp-cli` tools - so their **Install** stays disabled
  until you install Orcker's Composer. For the Laravel installer you can skip that
  entirely and keep using your external copy; WP-CLI has no such fallback, per
  the exception above.

::: tip How detection works
Because the daemon runs with a minimal environment, Orcker reads your login shell's
`PATH` to find tools your terminal can see (Homebrew, `fnm`, a global Composer
bin, …). It only looks **outside** its own `{data}/bin`, so a Orcker shim is never
mistaken for an external install.
:::

## How it works

The model mirrors [PHP versions](./php-versions) and [services](./services):

- **Self-contained binaries.** Each tool is a relocatable build - Node's tarball
  bundles `node` + `npm` + `npx`, Bun is a single binary, Composer is a phar run
  by Orcker's managed PHP. Nothing is compiled and nothing touches system paths.
- **Verified downloads.** Every artifact is checked against the publisher's
  `SHASUMS256.txt` (Node, Bun) or `composer.phar.sha256sum` (Composer) before it
  is installed.
- **Installed under Orcker's data dir.** Tools live in `{data}/tools/<id>/`
  (e.g. `~/Library/Application Support/io.orcker.Orcker/tools` on macOS), a sibling of
  your PHP installs - so a PHP update never disturbs them.
- **Exposed on `PATH`.** Their commands are symlinked into `{data}/bin`, the same
  directory that holds the `php`/`php<ver>` shims. Put that directory on your
  `PATH` once (see below) and `composer`, `node`, `npm`, `bun`, … just work.
- **Rootless.** Everything runs as your user, no elevation.

### Latest only

Orcker installs the **latest stable** release of each tool (the latest **LTS** for
Node). There is no per-project version picker - **Update** simply re-fetches the
current latest and replaces it in place. If you need to pin a specific Node
version per project, a system version manager like `nvm`/`fnm` is still the right
tool; Orcker's goal here is a good default that's always there.

## Put Orcker's bin directory on your PATH

The tool commands live in Orcker's `{data}/bin` directory. Installing your first
tool from the CLI **adds it to your shell automatically** - so usually there's
nothing to do. If you installed via the desktop app, or want to manage the entry
yourself, run it once:

```sh
orcker path install     # adds {data}/bin to your shell startup file
```

Open a new terminal afterwards (or `source` your shell file). Then:

```sh
which composer        # → …/io.orcker.Orcker/bin/composer
node --version
npm --version
bun --version
```

`orcker path install` writes a small, guarded block to your shell's startup file
(`.zshrc`, `.bashrc`/`.bash_profile`, or `config.fish`). `orcker path uninstall`
removes it; `orcker path print` shows the snippet without touching any file.

::: info Coexisting with Herd, Homebrew, or nvm
Orcker's `bin` directory is **prepended** to `PATH`, so its `node`/`composer` take
precedence over other copies on your machine. If you'd rather your existing
tools win, put their directories earlier in your shell file. Nothing Orcker
installs ever shadows a tool you didn't ask it to manage.
:::

## Composer needs PHP

Composer is a phar, so it runs under Orcker's managed PHP - `composer` resolves to
your [default PHP version](./php-versions). Install at least one PHP version
first (`orcker install php 8.4`); otherwise `composer` reports that no PHP is
available. Node and Bun are standalone and have no such dependency.

The `composer` shim always uses the **global default**, including inside a site
pinned to another version. To run Composer under a site's pinned version instead,
use `orcker exec composer …` - it runs the same phar under that site's PHP. See
[Site-aware CLI](./php-versions#site-aware-cli-orcker-exec-and-orcker-which). The
shim itself is unchanged.

::: tip ext-intl and friends
Orcker's PHP builds ship the **bulk** extension set, including
`intl`, `sodium`, `mysqli`, and more - so Composer packages that require them
install without extra steps. See [PHP Versions](./php-versions) for the bundled
extension list.
:::

## Where things live

| Path | Contents |
|---|---|
| `{data}/tools/composer/composer.phar` | The Composer phar. |
| `{data}/tools/node/node-<ver>-<os>-<arch>/` | The unpacked Node distribution. |
| `{data}/tools/bun/bun-<os>-<arch>/bun` | The Bun binary. |
| `{data}/tools/laravel/bin/laravel` | The Laravel installer (built via Composer). |
| `{data}/tools/wp-cli/vendor/wp-cli/wp-cli/php/boot-fs.php` | WP-CLI (built via Composer). |
| `{data}/bin/{composer,node,npm,npx,bun,bunx,laravel,wp}` | The `PATH` shims. |

`{data}` is Orcker's per-user data directory (`orcker status` and
`orcker path print` both show the exact path for your platform).

## See also

- [Tooling CLI reference](../reference/cli/tooling) - every command and flag.
- [PHP Versions](./php-versions) - the version model these tools follow.
- [Services & Databases](./services) - the same install-on-demand approach for
  databases and caches.
