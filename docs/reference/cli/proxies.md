# Proxies

A **reverse proxy** fronts an already-running service with a Orcker `.test`
address, so it gets the same DNS, HTTPS, and browser experience as a PHP site -
without Orcker running the service itself. Two shapes:

- A **whole-host proxy** maps a new host to a service: `reverb.test` → a Reverb,
  Node, Vite, or Docker service on some port.
- A **path rule** maps a path *on an existing site* to a service:
  `myapp.test/app` → a service, while every other path on `myapp.test` keeps
  being served by PHP. This is the same-origin setup Laravel Reverb wants
  (`wss://myapp.test/app`), so cookies and CORS just work.

`orcker proxy add`'s two forms are distinguished by argument count: two arguments
create a whole-host proxy, three attach a path rule to a site.

::: tip Looking for a path that maps to a *file* instead?
A proxy path rule forwards to a separate running service. To send unmatched URIs
under a prefix to a file **inside the site** - a nested `api/index.php`, or
`index.html` for a JavaScript SPA - use [`orcker route`](./routes) instead. When a
site has both on the same prefix, the proxy rule wins.
:::

| Command | Description | Example |
| --- | --- | --- |
| `orcker proxy add <NAME> <URL>` | Create a whole-host proxy (`<NAME>.test` → `<URL>`). | `orcker proxy add reverb http://localhost:8080` |
| `orcker proxy add <SITE> <PREFIX> <URL>` | Add a path rule: requests under `<PREFIX>` on `<SITE>.test` proxy to `<URL>`. | `orcker proxy add myapp /app http://127.0.0.1:8080` |
| `orcker proxy remove <NAME>` | Remove a whole-host proxy. | `orcker proxy remove reverb` |
| `orcker proxy remove <SITE> <PREFIX>` | Remove a path rule from a site. | `orcker proxy remove myapp /app` |
| `orcker proxy list` | List every whole-host proxy and per-site path rule. | `orcker proxy list` |

```sh
# Whole-host: front a Reverb server on its own .test domain
orcker proxy add reverb http://localhost:8080
curl http://reverb.test/          # → the Reverb service

# Serve it over HTTPS - a proxy secures exactly like a site
orcker secure reverb
curl https://reverb.test/

# Path rule (the Reverb same-origin case): /app on an existing Laravel site
orcker proxy add myapp /app http://127.0.0.1:8080
# https://myapp.test/        → PHP (Laravel)
# https://myapp.test/app     → Reverb (websockets included)

# List, then remove
orcker proxy list
orcker proxy remove myapp /app
orcker proxy remove reverb
```

A path rule inherits its parent site's TLS: securing `myapp` (`orcker secure
myapp`) also secures its `/app` rule. A whole-host proxy is secured on its own
name (`orcker secure reverb` / `orcker unsecure reverb`), exactly like a site.

## Names and domains

A whole-host proxy's name may be **dotted**, so the proxy's own address can be a
subdomain. Beyond that address, a proxy carries extra domains, subdomains, and
wildcards exactly as a site does, through the same
[`orcker domain`](./domains) commands - pass the proxy name where a site name
would go.

```sh
# A dotted name gives the proxy a subdomain address of its own
orcker proxy add api.account http://127.0.0.1:9011
curl http://api.account.test/

# Extra domains and wildcards, exactly as for a site
orcker proxy add account-dev http://127.0.0.1:48087
orcker domain add account-dev custom-domain.test
orcker domain add account-dev '*.account-dev.test'
orcker domain primary account-dev custom-domain.test
```

`orcker proxy list` shows a customized proxy's domains on an indented line beneath
it, marking the primary when you have set one away from the default
`<name>.<tld>`. `orcker domain list` remains site-only.

## Upstreams

The upstream `<URL>` is `http://host:port` or `https://host:port`. The port is
required for anything but the default (`80`/`443`). For an `https://` upstream,
Orcker verifies the certificate for a genuine public host, but **skips
verification for a local host** (`localhost`, a loopback/private IP, or a
`.test` name) - self-signed dev backends are the norm there.

The request path is passed to the upstream **unchanged** (an nginx
`proxy_pass` with no trailing path), so a path rule's prefix reaches the
upstream too: `myapp.test/app/foo` → `<URL>/app/foo`. Orcker preserves the
original `Host` header and adds the usual `X-Forwarded-For`,
`X-Forwarded-Proto`, `X-Forwarded-Host`, and `X-Real-IP` headers. Websocket
(`Connection: Upgrade`) traffic is tunnelled through, so Reverb/Vite HMR work.

If the upstream is down, a proxied request returns **`502 Bad Gateway`** rather
than hanging.

## How a request routes

For an incoming host, Orcker resolves it to a site or a whole-host proxy, then:

- a **whole-host proxy** forwards every request to its upstream (no PHP);
- a **site** with a matching **path rule** forwards that request to the rule's
  upstream; every other path is served by PHP as usual. Matching is
  longest-prefix with path boundaries, so `/app` matches `/app` and `/app/x`
  but **not** `/apple`.

If a whole-host proxy's name collides with a real site's apex, the site wins and
the proxy is dropped. Every **other** contested domain is settled one domain at a
time: a domain added explicitly beats a claim on a default apex, and among equal
claims the earlier claimant wins - every site is considered before any proxy, and
proxies among themselves in config order. A proxy that loses one domain keeps its
others. [`orcker doctor`](./diagnostics) reports each contested domain with its
winner and the claimants that lost it.

::: details Client-side validation & guards
`proxy add` validates the upstream URL and, for a path rule, that the prefix
begins with `/`, before connecting - a malformed value fails with a usage error
(exit code `2`). The daemon rejects a target that would **loop back into Orcker**:
a `.test` host, or a loopback host on one of Orcker's own bound proxy ports. A
proxy name that collides with an existing site or proxy is rejected as
"already exists".
:::

## See also

- [HTTPS & Certificates](../../guide/https) - how `secure` mints a trusted cert.
- [Reverse Proxies](../../guide/proxies) - the guide-level walkthrough.
- [Domains](./domains) - give a proxy or site more `.test` names.
