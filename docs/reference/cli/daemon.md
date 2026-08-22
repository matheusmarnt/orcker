# Daemon control

| Command | Description | Example |
| --- | --- | --- |
| `orcker restart daemon` | Restart the daemon itself. | `orcker restart daemon` |

```sh
orcker restart daemon
```

::: warning
`orcker restart daemon` briefly interrupts all sites, and this command itself, since it's a client of the daemon it's restarting.
:::

There is no `orcker start`/`orcker stop` subcommand: the daemon's lifecycle is managed by your OS (and started on demand). See [The Daemon](../../guide/daemon) for how `orckerd` is launched and supervised, and the [orckerd binary page](../../developer/binaries/orckerd) for internals.

`orcker update --yes` (see [Self-Update](./update)) also restarts the daemon, as the final step of installing a new Orcker version - that restart goes through the same OS service mechanics as `restart daemon`, not a fresh `orckerd` process started from scratch.
