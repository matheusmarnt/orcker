# Reverse Proxies

Orcker can put a `.test` address in front of a service it doesn't run itself - a
Reverb server, a Node or Vite dev server, a Docker container, anything already
listening on a port. The service gets Orcker's DNS, its trusted HTTPS, and a clean
`.test` hostname, with no extra config on the service's side.

There are two shapes, and you'll usually want the second for Laravel work:

- A **whole-host proxy** gives a service its own hostname: `reverb.test` → a
  service on `localhost:8080`.
- A **path rule** routes one path *on an existing site* to a service:
  `myapp.test/app` → a service, while every other path on `myapp.test` is still
  served by PHP. This keeps everything **same-origin**, which is exactly what
  Laravel Reverb needs (`wss://myapp.test/app`) so cookies and CORS work without
  a second domain.

Unlike [Herd](https://herd.laravel.com), which writes an nginx vhost, Orcker's
proxy is built into its own request path - there's nothing to configure and no
web server to reload.

## Whole-host proxies

Point a new `.test` host at a running service:

```sh
orcker proxy add reverb http://localhost:8080
```

Now `http://reverb.test/` reaches the service. Serve it over HTTPS the same way
you would a site - a proxy is secured on its own name:

```sh
orcker secure reverb
# https://reverb.test/  (trusted cert, HTTP redirects to HTTPS)
```

Remove it with `orcker proxy remove reverb`.

### Names and extra domains

A proxy's name may be **dotted**, so its own address can be a subdomain:

```sh
orcker proxy add api.account http://127.0.0.1:9011
# http://api.account.test/
```

Beyond that address, a whole-host proxy carries extra domains, subdomains, and
wildcards exactly as a site does - through the same [`orcker domain`](../reference/cli/domains)
commands, with the proxy name where a site name would go:

```sh
orcker proxy add account-dev http://127.0.0.1:48087
orcker domain add account-dev custom-domain.test
orcker domain add account-dev '*.account-dev.test'
orcker domain primary account-dev custom-domain.test
```

`orcker proxy list` shows a customised proxy's domains beneath it and marks the
primary; `orcker domain list` stays site-only. In the desktop app the same controls
are the **Manage domains** button on the Proxies page.

A domain can only be claimed by one site or proxy. If two claim the same one,
every **site** is considered before any proxy, and a domain added explicitly
beats a claim on a default apex - a proxy that loses one domain keeps its
others, and [`orcker doctor`](../reference/cli/diagnostics) reports each contested
domain with its winner.

## Path rules (the Reverb case)

Attach a path to an existing site. Say `myapp` is a Laravel app and Reverb is
running on `:8080`:

```sh
orcker proxy add myapp /app http://127.0.0.1:8080
```

- `https://myapp.test/` and everything else → **PHP** (Laravel), unchanged.
- `https://myapp.test/app` (and `/app/...`) → **Reverb**, websockets included.

The rule inherits the site's TLS, so once `myapp` is secured the `/app` path is
too - your JS client connects to `wss://myapp.test/app` on the same origin. The
full path is passed through to the upstream unchanged (`/app/...` reaches
Reverb as `/app/...`), which is what Reverb expects.

Remove a rule with `orcker proxy remove myapp /app`.

## Upstreams and headers

The upstream is `http://host:port` or `https://host:port`. For an `https://`
upstream, Orcker verifies the certificate for a genuine public host but **skips
verification for a local host** (`localhost`, a loopback/private IP, or a `.test`
name) - self-signed dev backends are the norm there.

Orcker preserves the original `Host` header (many upstreams key vhosts on it) and
adds `X-Forwarded-For`, `X-Forwarded-Proto`, `X-Forwarded-Host`, and `X-Real-IP`.
Websocket upgrades are tunnelled through. If the upstream is down, a request
returns **`502 Bad Gateway`** rather than hanging.

::: warning You can't proxy to Orcker itself
A target that points back into Orcker - a `.test` host, or `localhost` on the port
Orcker's own proxy is listening on - is rejected, because it would loop forever.
Point the target at the service's real port instead. In rootless mode Orcker binds
`8080`/`8443`, so a dev server on one of those ports will need to move.
:::

## Listing what's configured

```sh
orcker proxy list
```

shows every whole-host proxy and every per-site path rule. `orcker --json proxy
list` gives the same data as JSON for scripting or the desktop app.

For the full command surface, flags, and validation rules, see the
[Proxies CLI reference](../reference/cli/proxies).
