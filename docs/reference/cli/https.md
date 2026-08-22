# HTTPS

`secure` promotes a parked site to a linked entry and serves it over HTTPS using the local certificate authority; `unsecure` stops serving it over HTTPS. See [HTTPS & Certificates](../../guide/https).

| Command | Description | Example |
| --- | --- | --- |
| `orcker secure <NAME>` | Serve a site over HTTPS (promotes a parked site to a linked entry). | `orcker secure blog` |
| `orcker unsecure <NAME>` | Stop serving a site over HTTPS. | `orcker unsecure blog` |

```sh
orcker secure blog      # https://blog.test is now served with a trusted cert
orcker unsecure blog    # back to http only
```

::: tip
For the browser to trust the certificate, the local CA must be installed in your OS trust store. Run `sudo orcker elevate trust` once (see [Elevation](./elevation)).
:::
