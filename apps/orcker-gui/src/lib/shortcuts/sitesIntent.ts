/**
 * A one-shot request to open a Sites-page dialog from elsewhere (the command
 * palette, a shortcut, or the system-tray Link/Park items). The Link/Park
 * commands set this, then navigate to `/sites`; SitesView consumes and clears
 * it on mount (or while already mounted).
 *
 * Module-level singleton, like `useViewActions` - and per-webview, so the
 * standalone mails window holds its own (always-null) copy.
 */
import { ref } from "vue";

export type SitesIntent = "link" | "park";

export const sitesIntent = ref<SitesIntent | null>(null);
