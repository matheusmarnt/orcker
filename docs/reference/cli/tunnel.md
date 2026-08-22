# Tunnel

The `orcker tunnel` commands publish a local site to the public internet through
[Cloudflare Tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/)
(`cloudflared`). The connection is outbound-only - no inbound ports, no `sudo`.
The [Sharing Sites guide](../../guide/sharing) covers the model and the two tiers
(quick tunnels and named tunnels); this page is the command reference.

::: info cloudflared is detected on PATH, or downloaded on demand
The first share needs the `cloudflared` binary. Orcker looks for one already on
your `PATH` first and uses it if it's version 2023.3.0 or newer; otherwise
`orcker tunnel install` fetches the official static build (checksum-verified) into
the daemon's data directory - it is not bundled and never touches your system
unless you already had a compatible `cloudflared` installed. Quick share needs no
Cloudflare account; named tunnels need a one-time `orcker tunnel login`.
:::

## Commands

| Command | Description |
| --- | --- |
| `orcker tunnel install` | Download the `cloudflared` binary (required before sharing). |
| `orcker tunnel share <SITE>` | Share a site via a Quick Tunnel (random `*.trycloudflare.com` URL). |
| `orcker tunnel stop <SITE>` | Stop sharing a site. |
| `orcker tunnel status` | Show live tunnels and `cloudflared` install status. |
| `orcker tunnel login` | Log in to a Cloudflare account (opens a browser) for named tunnels. |
| `orcker tunnel create <NAME>` | Create a named tunnel on the logged-in account. |
| `orcker tunnel delete <NAME>` | Delete a named tunnel and forget it locally. |
| `orcker tunnel list` | List the named tunnel, exposed sites, and authorized domain. |
| `orcker tunnel route <TUNNEL> <HOSTNAME>` | Create the proxied DNS record for a hostname. |
| `orcker tunnel set-host <SITE> [HOSTNAME] [--clear]` | Set or clear a site's public hostname. |
| `orcker tunnel publish` | Start the named tunnel, exposing every site with a hostname set. |
| `orcker tunnel unpublish` | Stop the named tunnel (takes every named site offline). |

```sh
orcker tunnel install
orcker tunnel share app
orcker tunnel status
orcker tunnel stop app
```

## Quick tunnels

### `orcker tunnel install`

Downloads and installs `cloudflared`. Idempotent; safe to re-run. Needed once
before any share, unless a compatible `cloudflared` is already on `PATH` (see
above). Re-running it always switches Orcker back to the managed copy, even if a
`PATH` binary was previously in use.

### `orcker tunnel share <SITE>`

Publishes `<SITE>` (a site name like `app`, or its host `app.test`) at a random
`https://<name>.trycloudflare.com` URL and prints it. The URL is temporary - it
changes on the next share and is torn down on `stop` or daemon shutdown. Requires
`cloudflared` to be installed.

### `orcker tunnel stop <SITE>`

Tears down the site's tunnel. No-op if it isn't shared.

### `orcker tunnel status`

Lists live tunnels (site, kind, state, URL/hostname) and whether `cloudflared` is
installed and logged in, plus its version and source: `managed` (Orcker's own
download) or `system` (found on your `PATH`) in `--json` output.

## Named tunnels

Named tunnels serve a stable hostname on a domain you manage in Cloudflare. They
use one consolidated tunnel for every exposed site.

### `orcker tunnel login`

Opens a browser to authorize Orcker for a Cloudflare zone. The account certificate
is stored in the daemon's data directory. Run once per machine.

### `orcker tunnel create <NAME>`

Creates a named tunnel on the logged-in account and records it locally. Orcker
supports one named tunnel at a time; `delete` the existing one before creating a
differently-named tunnel.

### `orcker tunnel set-host <SITE> [HOSTNAME] [--clear]`

Maps `<SITE>` to a public `HOSTNAME` on your authorized domain: it creates the
proxied DNS record and records the mapping so the consolidated tunnel serves it.
`HOSTNAME` is **required unless** `--clear` is given - `--clear` removes the
site's mapping (and the two are mutually exclusive, so forgetting the hostname
can't silently clear it).

```sh
orcker tunnel set-host app app.example.com
orcker tunnel set-host app --clear
```

### `orcker tunnel publish` / `orcker tunnel unpublish`

`publish` (re)starts the consolidated named tunnel, serving every site that has a
hostname set. `unpublish` stops it, taking all named sites offline. Both require
login and a created tunnel.

### `orcker tunnel route <TUNNEL> <HOSTNAME>`

Creates the proxied CNAME routing `HOSTNAME` to `TUNNEL` on your Cloudflare zone.
`set-host` does this for you; `route` is the lower-level primitive.

### `orcker tunnel list`

Shows the recorded named tunnel, the per-site hostname mappings that are exposed,
and the **authorized domain** resolved from your account.

### `orcker tunnel delete <NAME>`

Deletes the named tunnel from your Cloudflare account and forgets it locally:
stops the process, removes the credentials, and clears the exposed-site mappings.
Only the one configured tunnel may be deleted. The DNS records created by
`set-host`/`route` remain on your account - remove them in the Cloudflare
dashboard if you no longer want them.

::: tip Desktop app
The desktop app's **Share** page (under **Integrations**) drives all of this with
a searchable site picker, a per-site expose toggle, and automatic start/stop. See
the [Sharing Sites guide](../../guide/sharing).
:::
