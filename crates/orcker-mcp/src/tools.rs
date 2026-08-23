//! The tool catalog: what agents may call, and how each call becomes exactly
//! one [`orcker_ipc::Request`].
//!
//! The catalog is curated rather than a mirror of the IPC surface. Two reasons:
//! a large tool list measurably degrades an agent's tool selection and costs
//! tokens on every turn, and the IPC surface includes destructive operations
//! (drop a database, uninstall PHP, unlink a site, clear captured mail) that an
//! unattended agent has no business reaching. Removing a proxy, a proxy rule, or
//! an added domain *is* included: those revert reversible config the agent
//! typically added itself, and the daemon already refuses to remove a site's
//! last domain.
//!
//! The other axis is **egress**: nothing here may send a developer's data off
//! the machine. That is why tunnels are absent and why proxy upstreams are
//! restricted to loopback (see [`req_local_url`]) even though the CLI allows
//! any host - an agent can be talked into things a user cannot.
//!
//! The catalog is pinned by `tests/tools.rs`. Treat a failure there as a
//! contract alarm: this list is what agents build their calls against.

use std::path::PathBuf;

use orcker_ipc::Request;
use serde_json::{json, Value};

use crate::ArgError;

/// One catalog entry. Schemas live in [`schema_for`] rather than here so they
/// can be built with `json!` instead of parsed from text at runtime.
struct ToolDef {
    name: &'static str,
    description: &'static str,
}

/// The catalog, in the order agents see it.
const TOOLS: &[ToolDef] = &[
    ToolDef {
        name: "list_sites",
        description: "List every site Orcker serves, with its domains, PHP version, and HTTPS state.",
    },
    ToolDef {
        name: "link_site",
        description: "Serve an existing project directory as <name>.test.",
    },
    ToolDef {
        name: "park_directory",
        description: "Park a directory so each child folder is served as its own .test site.",
    },
    ToolDef {
        name: "list_parked",
        description: "List the parked directory roots.",
    },
    ToolDef {
        name: "set_site_secure",
        description: "Turn HTTPS on or off for one site.",
    },
    ToolDef {
        name: "add_domain",
        description:
            "Add an extra domain or wildcard to a site or whole-host proxy, e.g. api.foo.test.",
    },
    ToolDef {
        name: "remove_domain",
        description:
            "Remove a domain from a site or whole-host proxy (the last domain cannot be removed).",
    },
    ToolDef {
        name: "add_proxy",
        description: "Route a whole host, <name>.test, to a local upstream URL.",
    },
    ToolDef {
        name: "remove_proxy",
        description: "Remove a whole-host proxy.",
    },
    ToolDef {
        name: "add_proxy_rule",
        description: "Proxy one path prefix of a site to a local upstream URL, e.g. app.test/reverb.",
    },
    ToolDef {
        name: "remove_proxy_rule",
        description: "Remove a path-prefix proxy rule from a site.",
    },
    ToolDef {
        name: "list_proxies",
        description: "List whole-host proxies and path-prefix proxy rules.",
    },
    ToolDef {
        name: "set_mail_enabled",
        description: "Turn Orcker's mail-capture SMTP sink on or off (takes effect on the next daemon restart).",
    },
    ToolDef {
        name: "list_mails",
        description: "List captured emails, newest first, as metadata only.",
    },
    ToolDef {
        name: "get_mail",
        description: "Read one captured email in full, including its decoded text and HTML bodies.",
    },
    ToolDef {
        name: "status",
        description: "Health snapshot: ports, DNS, CA trust, PHP pools, services, and mail capture.",
    },
    ToolDef {
        name: "diagnose",
        description: "Run Orcker's doctor checks and report findings with their remedies.",
    },
    ToolDef {
        name: "job_status",
        description: "Poll a background job for state, phase, and new log lines; jobs are in-memory, so an unknown job_id means it expired or the daemon restarted.",
    },
];

/// Build the `tools/list` result. The `inputSchema` key is camelCase per the MCP
/// spec; the golden test pins it.
pub(crate) fn list_result() -> Value {
    let tools: Vec<Value> = TOOLS
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": schema_for(t.name),
            })
        })
        .collect();
    json!({ "tools": tools })
}

/// Look a tool up and turn its arguments into the daemon request it maps to.
/// Returns the catalog name alongside the request so the caller can render the
/// answer without re-matching the client's (unvalidated) string.
pub(crate) fn build(name: &str, args: &Value) -> Result<(&'static str, Request), ArgError> {
    let def = TOOLS
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| ArgError::UnknownTool(name.to_owned()))?;
    let request = build_request(def.name, args)?;
    Ok((def.name, request))
}

/// Build the request one catalog tool maps to.
///
/// The trailing `UnknownTool` arm exists only to make the match total: [`build`]
/// resolves the name against [`TOOLS`] first, so it is reachable only if a
/// catalog entry has no arm here - which `tests/tools.rs::every_tool_builds_a_request`
/// rules out.
#[allow(clippy::too_many_lines)]
fn build_request(name: &'static str, args: &Value) -> Result<Request, ArgError> {
    let request = match name {
        "list_sites" => Request::ListSites,
        "link_site" => Request::Link {
            name: req_str(args, "name")?,
            path: PathBuf::from(req_str(args, "path")?),
        },
        "park_directory" => Request::Park {
            path: PathBuf::from(req_str(args, "path")?),
        },
        "list_parked" => Request::ListParked,
        "set_site_secure" => Request::SetSecure {
            name: req_str(args, "name")?,
            secure: req_bool(args, "secure")?,
        },
        "add_domain" => Request::AddDomain {
            name: req_str(args, "name")?,
            domain: req_str(args, "domain")?,
        },
        "remove_domain" => Request::RemoveDomain {
            name: req_str(args, "name")?,
            domain: req_str(args, "domain")?,
        },
        "add_proxy" => Request::AddProxy {
            name: req_str(args, "name")?,
            url: req_local_url(args, "url")?,
        },
        "remove_proxy" => Request::RemoveProxy {
            name: req_str(args, "name")?,
        },
        "add_proxy_rule" => Request::AddProxyRule {
            site: req_str(args, "site")?,
            prefix: req_str(args, "prefix")?,
            url: req_local_url(args, "url")?,
        },
        "remove_proxy_rule" => Request::RemoveProxyRule {
            site: req_str(args, "site")?,
            prefix: req_str(args, "prefix")?,
        },
        "list_proxies" => Request::ListProxies,
        "set_mail_enabled" => Request::SetMailEnabled {
            enabled: req_bool(args, "enabled")?,
        },
        "list_mails" => Request::ListMails,
        "get_mail" => Request::GetMail {
            id: req_str(args, "id")?,
        },
        "status" => Request::Status,
        "diagnose" => Request::Diagnose,
        "job_status" => Request::JobStatus {
            job_id: req_str(args, "job_id")?,
            cursor: opt_u64(args, "cursor", 0)?,
        },
        other => return Err(ArgError::UnknownTool(other.to_owned())),
    };
    Ok(request)
}

#[allow(clippy::too_many_lines)]
fn schema_for(name: &str) -> Value {
    match name {
        "link_site" => schema(
            &[
                ("name", string_prop("Site name: one DNS label, becomes <name>.test")),
                ("path", string_prop("Absolute path of the existing project directory")),
            ],
            &["name", "path"],
        ),
        "park_directory" => schema(
            &[("path", string_prop("Absolute path of the directory to park"))],
            &["path"],
        ),
        "set_site_secure" => schema(
            &[
                ("name", string_prop("Site name")),
                ("secure", bool_prop("True to serve over HTTPS, false for HTTP")),
            ],
            &["name", "secure"],
        ),
        "add_domain" | "remove_domain" => schema(
            &[
                ("name", string_prop("Site or proxy name")),
                ("domain", string_prop("Full domain, e.g. api.foo.test or *.foo.test")),
            ],
            &["name", "domain"],
        ),
        "add_proxy" => schema(
            &[
                ("name", string_prop("Host label: serves <name>.test")),
                ("url", string_prop("Upstream URL on this machine, e.g. http://127.0.0.1:8000; remote hosts are refused")),
            ],
            &["name", "url"],
        ),
        "remove_proxy" => schema(
            &[("name", string_prop("Host label of the proxy to remove"))],
            &["name"],
        ),
        "add_proxy_rule" => schema(
            &[
                ("site", string_prop("Site name the rule applies to")),
                ("prefix", string_prop("Path prefix starting with /, e.g. /reverb")),
                ("url", string_prop("Upstream URL on this machine, e.g. http://127.0.0.1:8080; remote hosts are refused")),
            ],
            &["site", "prefix", "url"],
        ),
        "remove_proxy_rule" => schema(
            &[
                ("site", string_prop("Site name the rule applies to")),
                ("prefix", string_prop("Path prefix of the rule to remove")),
            ],
            &["site", "prefix"],
        ),
        "set_mail_enabled" => schema(
            &[("enabled", bool_prop("True to enable, false to disable"))],
            &["enabled"],
        ),
        "get_mail" => schema(
            &[("id", string_prop("Mail id from list_mails"))],
            &["id"],
        ),
        "job_status" => schema(
            &[
                ("job_id", string_prop("Job id returned by a tool that starts background work")),
                ("cursor", int_prop("Log cursor: pass the next_cursor from the previous poll to get only new lines (default 0)")),
            ],
            &["job_id"],
        ),
        _ => no_args(),
    }
}

fn schema(props: &[(&str, Value)], required: &[&str]) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in props {
        map.insert((*key).to_owned(), value.clone());
    }
    let mut root = serde_json::Map::new();
    root.insert("type".to_owned(), json!("object"));
    root.insert("properties".to_owned(), Value::Object(map));
    if !required.is_empty() {
        root.insert("required".to_owned(), json!(required));
    }
    root.insert("additionalProperties".to_owned(), json!(false));
    Value::Object(root)
}

fn no_args() -> Value {
    json!({ "type": "object", "properties": {}, "additionalProperties": false })
}

fn string_prop(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

fn bool_prop(description: &str) -> Value {
    json!({ "type": "boolean", "description": description })
}

fn int_prop(description: &str) -> Value {
    json!({ "type": "integer", "minimum": 0, "description": description })
}

fn req_str(args: &Value, name: &'static str) -> Result<String, ArgError> {
    match args.get(name) {
        None | Some(Value::Null) => Err(ArgError::Missing(name)),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(ArgError::Type {
            name,
            expected: "a string",
        }),
    }
}

fn req_bool(args: &Value, name: &'static str) -> Result<bool, ArgError> {
    match args.get(name) {
        None | Some(Value::Null) => Err(ArgError::Missing(name)),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(ArgError::Type {
            name,
            expected: "a boolean",
        }),
    }
}

fn opt_u64(args: &Value, name: &'static str, default: u64) -> Result<u64, ArgError> {
    match args.get(name) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v.as_u64().ok_or(ArgError::Type {
            name,
            expected: "a non-negative integer",
        }),
    }
}

/// A proxy upstream, restricted to this machine.
///
/// Proxies are the one thing in the catalog whose blast radius leaves the box.
/// A rule pointing `app.test/api` at a remote host makes the developer's browser
/// send that site's cookies and bearer tokens there, over an origin their system
/// already trusts, with no TLS warning - and the rule outlives the agent session
/// that added it. An agent acting on instructions injected into a page or file
/// it read is exactly the case that matters, and it is the same egress risk that
/// kept tunnels out of the catalog, so the two are treated alike.
///
/// This costs no real capability: these tools exist to wire up local dev
/// services (Vite, Reverb, a local API), which is what their descriptions
/// advertise. A genuine off-box upstream is still one `orcker proxy add` away, by
/// a human who meant it.
fn req_local_url(args: &Value, name: &'static str) -> Result<String, ArgError> {
    let raw = req_str(args, name)?;
    let target =
        orcker_core::UpstreamTarget::from_url_str(&raw).map_err(|e| ArgError::Invalid {
            name,
            reason: e.to_string(),
        })?;
    if !is_loopback_host(target.host()) {
        return Err(ArgError::Invalid {
            name,
            reason: format!(
                "`{}` is not a loopback address. Proxy upstreams must name this machine \
                 explicitly: localhost, 127.0.0.0/8, or ::1. A dev server that advertises \
                 0.0.0.0:PORT is reachable at 127.0.0.1:PORT. If a genuinely remote upstream is \
                 wanted, ask the user to add it themselves with `orcker proxy add`.",
                target.host()
            ),
        });
    }
    Ok(raw)
}

/// Whether a host names this machine.
///
/// Matches the literal host rather than resolving it: resolution is I/O (which
/// this crate does not do), and a name that resolves to `127.0.0.1` today can
/// resolve elsewhere tomorrow. The accepted set is therefore closed and exact -
/// `localhost`, `127.0.0.0/8`, `::1` - with no suffix or substring matching that
/// `localhost.attacker.com` could slip through.
///
/// Anything it cannot recognise is refused, so the gate never has to out-guess
/// the resolver. That costs a few odd-but-local spellings (`0.0.0.0`, `127.1`,
/// `::ffff:127.0.0.1`), which fail in the safe direction and are covered by the
/// error text.
///
/// Takes the host as [`orcker_core::UpstreamTarget`] parsed it: already
/// lowercased and with IPv6 brackets stripped, so this sees `localhost` and
/// `::1`, never `LOCALHOST` or `[::1]`.
fn is_loopback_host(host: &str) -> bool {
    host == "localhost"
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}
