# Sites

A directory tree is served in one of two ways: **park** a parent directory so each child folder automatically becomes a `<child>.test` site, or **link** a single directory under an explicit name. See the [Sites guide](../../guide/sites) for the full model.

| Command | Description | Example |
| --- | --- | --- |
| `orcker sites` | List every parked or linked site. | `orcker sites` |
| `orcker park <PATH>` | Park a directory: each of its child directories becomes a `.test` site. | `orcker park ~/Sites` |
| `orcker unpark <PATH>` | Un-park a directory so its children stop being served. Linked sites are untouched. | `orcker unpark ~/Sites` |
| `orcker link` | Link the current directory, named after its folder. | `orcker link` |
| `orcker link <NAME>` | Link the current directory under an explicit name. | `orcker link blog` |
| `orcker link <PATH>` | Link a directory, named after its folder. | `orcker link ~/code/blog` |
| `orcker link <NAME> <PATH>` | Link a directory under an explicit name. | `orcker link blog ~/code/blog` |
| `orcker unlink <NAME>` | Remove a linked site by name. | `orcker unlink blog` |
| `orcker root <NAME> <PATH>` | Set the served directory (web root) for a site, relative to its folder. | `orcker root blog public` |
| `orcker root <NAME> --auto` | Reset a site to automatic web-root detection. | `orcker root blog --auto` |

```sh
# Park a folder of projects: every subdirectory is reachable at <name>.test
orcker park ~/Sites

# Link one project under a specific name (serves https://blog.test once secured)
orcker link blog ~/code/blog

# Link the current project, named after its folder
cd ~/code/blog && orcker link

# See everything orcker is serving
orcker sites
```

`orcker sites` prints a table with the columns `NAME`, `KIND` (`parked` or `linked`), `PHP`, `SECURE`, `SERVED`, and `DOCROOT`. `SERVED` is the web root relative to the document root (`/` means the project root itself is served). When there are no sites it prints `no sites`. A `DOMAIN` column appears only when at least one site has a customised primary domain or a shadowed apex; each cell holds the site's primary FQDN, or `apex shadowed by <site>` when another site claims its apex, or `-`. Use [`orcker domain list`](./domains) to see the full per-site domain set (including subdomains and wildcards).

::: details How site names are validated
`link`, `unlink`, `secure`, `unsecure`, and `root` validate the name client-side before connecting: a name must be a single valid DNS label. A bad name (e.g. `bad name` or `bad/name`) fails immediately with a usage error and exit code `2`, before any request reaches the daemon.

`link` accepts a single positional argument as either a name or a path: an argument containing a path separator (or `.`/`..`) is treated as a directory, and the site name is derived from its folder name (lowercased, with runs of invalid characters collapsed to a single `-`); a bare word is always treated as a name, even if a same-named subdirectory happens to exist. With no arguments at all, the current directory is linked and named after its own folder.
:::

::: tip Web root detection
Orcker auto-detects the directory each site is served from (e.g. `public/` for Laravel, the project root for WordPress). For a **parked** site it re-detects continuously as the project changes. For a **linked** site detection runs once, when the site is first linked; it isn't re-run automatically afterward. `orcker root <name> <path>` pins it explicitly for either kind. `orcker root <name> --auto` (or with no path) returns to auto-detection: for a linked site this re-runs the one-shot detection immediately and pins the fresh result; for a parked site it clears the pin and hands the site back to the continuous watched detection. The path must resolve to a directory inside the site's folder. See the [Sites guide](../../guide/sites#web-root-the-served-directory).
:::

::: warning About `unpark`
The daemon stores parked roots by their canonical path and matches `unpark` against that stored string **exactly**. `orcker` best-effort canonicalises the path you type (resolving symlinks and relative paths) so it matches. If the directory has been deleted from disk it can't be canonicalised, so pass the exact stored path instead. Run `orcker list parked` to see the canonical paths the daemon holds, including empty roots that produce no sites and therefore don't show up in `orcker sites`.
:::
