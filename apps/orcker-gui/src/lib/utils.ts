import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/** shadcn-vue's `cn`: merge conditional class lists, de-duping Tailwind utils. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/** Humanise a duration given in whole seconds (e.g. `90061` -> `1d 1h 1m`). */
export function humaniseUptime(secs: number): string {
  if (!Number.isFinite(secs) || secs < 0) return "-";
  const d = Math.floor(secs / 86_400);
  const h = Math.floor((secs % 86_400) / 3_600);
  const m = Math.floor((secs % 3_600) / 60);
  const s = secs % 60;
  const parts: string[] = [];
  if (d) parts.push(`${d}d`);
  if (h) parts.push(`${h}h`);
  if (m) parts.push(`${m}m`);
  if (!d && !h) parts.push(`${s}s`);
  return parts.join(" ");
}

/** Render a past Unix-epoch (seconds) as a coarse, single-unit "… ago" string
 *  (e.g. "5 minutes ago", "2 hours ago", "3 days ago"). */
export function humaniseAgo(epochSecs: number, nowSecs: number = Date.now() / 1000): string {
  const diff = Math.floor(nowSecs - epochSecs);
  if (!Number.isFinite(diff) || diff < 45) return "just now";
  // Floor within each unit so age isn't overstated (e.g. 59m31s stays minutes,
  // 23h31m stays hours) rather than rounding up into the next unit. `mins` is
  // clamped to >= 1 since the sub-45s window is already "just now".
  const mins = Math.max(1, Math.floor(diff / 60));
  if (mins < 60) return `${mins} minute${mins === 1 ? "" : "s"} ago`;
  const hours = Math.floor(diff / 3_600);
  if (hours < 24) return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  const days = Math.floor(diff / 86_400);
  return `${days} day${days === 1 ? "" : "s"} ago`;
}

/** Render bytes as a short human string (e.g. `1536` -> `1.5 MB` base-2). */
export function humaniseBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes)) return "-";
  const units = ["B", "KB", "MB", "GB"];
  let v = bytes;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u += 1;
  }
  return `${u === 0 ? v : v.toFixed(1)} ${units[u]}`;
}
