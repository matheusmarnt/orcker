import type { Site, StatusReport } from "@/ipc/types";

/** The minimal site shape the URL helpers need - satisfied by a full `Site` and
 *  by the create wizard's in-progress form (which has no real `Site` yet).
 *  `primary_domain` (a full FQDN) is honoured as the site's address when set;
 *  otherwise the address is synthesized as `{name}.{tld}`. */
export type SiteLike = Pick<Site, "name" | "secure"> & { primary_domain?: string };

/** The site's address host (no scheme/port): its primary domain FQDN when set,
 *  else `{name}.{tld}`. */
function siteHost(s: SiteLike, tld: string): string {
  return s.primary_domain ?? `${s.name}.${tld}`;
}

/**
 * True when `.test` resolution is unavailable, so sites must be reached via the
 * `http://localhost/~{domain}` fallback rather than their `.test` domain. This
 * covers both the OS resolver not being active (`resolver_installed` is only
 * "on" when strictly `true`, tri-state aware) *and* the daemon failing to bind
 * its DNS responder port (`dns_unbound` set) - in which case names won't resolve
 * through Orcker even when the resolver is installed.
 */
export function isUnbound(report: StatusReport | null | undefined): boolean {
  return report?.resolver_installed !== true || report?.dns_unbound != null;
}

interface UnboundOpts {
  httpBound: number | undefined;
}

/** The bound HTTP port, or the `8080` rootless fallback. Uses a truthiness check
 *  rather than `?? 8080` so the daemon's degraded-mode `bound = 0` (couldn't bind
 *  the web ports) also falls back instead of producing a malformed `:0` URL. */
function boundHttpPortOr8080(bound: number | undefined): number {
  return bound && bound > 0 ? bound : 8080;
}

/**
 * The `http://localhost/~{host}` URL used when the resolver is off, where `host`
 * is the site's full domain FQDN (its primary domain). Always plain http (there
 * is no localhost cert), and the port is omitted when it is the default 80.
 */
export function unboundUrlFor(host: string, opts: UnboundOpts): string {
  const port = boundHttpPortOr8080(opts.httpBound);
  const portPart = port === 80 ? "" : `:${port}`;
  return `http://localhost${portPart}/~${host}`;
}

/**
 * Browser URL for a site's "Open" action. When the resolver is active this is
 * the site's `.test` domain (honouring scheme + bound port); when it is off,
 * the localhost `/~` fallback (forced http, `secure` ignored).
 */
export function siteUrl(s: SiteLike, report: StatusReport | null | undefined): string {
  const tld = report?.tld ?? "test";
  const host = siteHost(s, tld);
  if (isUnbound(report)) {
    return unboundUrlFor(host, { httpBound: report?.http.bound });
  }
  const scheme = s.secure ? "https" : "http";
  const bound = s.secure ? report?.https.bound : report?.http.bound;
  const dflt = s.secure ? 443 : 80;
  const redirected = report?.port_redirect === true;
  const port = !redirected && bound && bound !== dflt ? `:${bound}` : "";
  return `${scheme}://${host}${port}`;
}

/**
 * The WP Admin URL for a WordPress site - the site's own URL plus
 * `/wp-admin/`. Never pre-authenticated: this opens the ordinary WordPress
 * login screen. The one-click variant needed a daemon handler SPEC-0002
 * removed. `siteUrl` never returns a trailing slash in either branch, so
 * straight concatenation is safe here.
 */
export function wpAdminUrl(s: SiteLike, report: StatusReport | null | undefined): string {
  return `${siteUrl(s, report)}/wp-admin/`;
}

/**
 * Tooltip / aria text for an "Open" affordance. Appends the http-only caveat
 * when the resolver is off (the site is reached via the localhost `/~`
 * fallback). Shared so every Open affordance shows the same target + caveat.
 */
export function openTitle(s: SiteLike, report: StatusReport | null | undefined): string {
  const url = siteUrl(s, report);
  return isUnbound(report)
    ? `Open ${url} - served over http://localhost (forced-HTTPS sites may not load)`
    : `Open ${url}`;
}
