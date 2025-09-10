// Simple runtime check for Tauri environment
type TauriGlobals = {
  __TAURI_IPC__?: unknown
  __TAURI__?: unknown
  __TAURI_INTERNALS__?: unknown
}

export function isTauri(): boolean {
  // Tauri v1 exposes `__TAURI_IPC__` (function)
  // Tauri v2 may expose `__TAURI__` globals instead
  if (typeof window === "undefined") return false;
  const w = window as unknown as TauriGlobals;
  return (
    typeof w.__TAURI_IPC__ === "function" ||
    typeof w.__TAURI__ !== "undefined" ||
    typeof w.__TAURI_INTERNALS__ !== "undefined"
  );
}
