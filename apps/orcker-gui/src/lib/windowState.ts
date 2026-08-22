/**
 * Return whether the host exposes a reliable maximized state for the
 * decorationless Orcker window.
 */
export function supportsReliableMaximizedState(platform: string): boolean {
  return platform === "linux" || platform === "windows";
}
