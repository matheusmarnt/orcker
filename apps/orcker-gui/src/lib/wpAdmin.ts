import { openInBrowser } from "@/ipc/client";
import { wpAdminUrl } from "@/lib/siteUrl";
import type { SiteEntry, StatusReport } from "@/ipc/types";

/**
 * Open a site's WP Admin at the plain (not signed-in) link. Never blocks or
 * surfaces an error, just silently degrades.
 *
 * The one-click pre-authenticated variant went with SPEC-0002's native runtime:
 * minting a login token needed a daemon handler that no longer exists. WordPress
 * support returns over containers under PRD FR-020.
 *
 * Shared by the site card's "WPA" chip and the site details sidebar so both
 * entry points behave identically.
 */
export async function openWpAdmin(
  site: SiteEntry,
  report: StatusReport | null | undefined,
): Promise<void> {
  try {
    await openInBrowser(wpAdminUrl(site, report));
  } catch {
    /* nothing left to fall back to */
  }
}
