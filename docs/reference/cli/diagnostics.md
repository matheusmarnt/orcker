# Status & Diagnostics

| Command | Description | Example |
| --- | --- | --- |
| `orcker ping` | Check that the daemon is alive (prints `pong`). | `orcker ping` |
| `orcker status` | Show a snapshot of daemon, proxy, DNS, ports, CA, and PHP health. | `orcker status` |
| `orcker doctor` | Diagnose common problems and report findings. | `orcker doctor` |
| `orcker doctor fix` | Attempt safe, unprivileged repairs (e.g. restart a crashed FPM pool). | `orcker doctor fix` |

```sh
orcker ping            # pong, if the daemon is up
orcker status          # one-screen health snapshot
orcker doctor          # report problems and remedies
orcker doctor fix      # apply safe automatic fixes, then list what still needs you
```

`orcker status` reports the daemon PID, uptime and RSS, version, TLD, the bound HTTP/HTTPS ports (flagging rootless fallback or an active macOS pf redirect), the DNS responder address, CA trust state and path, resolver install state, load average, site counts, and a per-version PHP pool listing (state, PID, listen socket, RSS, available update).

`orcker doctor` prints each finding with a severity mark (`✓` ok, `⚠` warn, `✗` fail), a detail line, and a `→` remedy where one exists. `orcker doctor fix` first lists what it applied, then what still needs manual attention.

One such finding is `DomainShadowed` (a `Warn`): two sites claim the same domain, so one site's apex is shadowed by the other. The remedy is to `orcker domain remove` the duplicate or `orcker domain primary` the shadowed site onto a free domain.

::: info Exit codes for diagnostics
`orcker doctor` (and `orcker doctor fix`) exit `1` if any finding is `Fail` severity, otherwise `0`. A `Warn` alone does **not** fail the exit code. This holds in both human and `--json` modes, so doctor is safe to use in CI gates.

If the daemon is unreachable, `orcker doctor` is special-cased: instead of the generic "daemon unreachable" error, it surfaces a synthetic `Daemon not running` **Fail** finding and exits `1`, so a down daemon shows up as a doctor failure, consistently across `--json` and the exit code.
:::

See the [Diagnostics guide](../../guide/diagnostics) for an explanation of each check.
