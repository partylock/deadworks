import { PARTYLOCK_PROD_API_BASE } from "./partylock-api";

/** Set once at startup from Rust `is_debug_build` (true for `tauri dev`). */
let launcherDebugBuild: boolean | null = null;

export function setLauncherDebugBuild(value: boolean): void {
  launcherDebugBuild = value;
}

export function isLauncherDebugBuild(): boolean {
  return launcherDebugBuild === true;
}

/** True when this build/session should talk to localhost:3001. */
export function isLauncherLocalMode(): boolean {
  if (launcherDebugBuild === true) return true;
  if (import.meta.env.VITE_PARTYLOCK_FORCE_LOCAL === "true") return true;
  if (import.meta.env.DEV) return true;
  if (import.meta.env.MODE === "development") return true;

  if (typeof window !== "undefined") {
    const { hostname, port } = window.location;
    // Vite dev server — not tauri.localhost (packaged prod webview).
    if (hostname === "localhost" || hostname === "127.0.0.1") {
      return port === "1420";
    }
  }

  return false;
}

/** @deprecated use isLauncherLocalMode */
export function isTauriDevSession(): boolean {
  return isLauncherLocalMode();
}

export function resolveApiEndpoint(apiEndpoint: string): string {
  if (isLauncherLocalMode()) return "local";
  return apiEndpoint === "local" ? "local" : "prod";
}

export function getApiBaseUrl(apiEndpoint: string): string {
  if (resolveApiEndpoint(apiEndpoint) === "local") {
    return "http://localhost:3001/api/v1";
  }
  return import.meta.env.VITE_PARTYLOCK_API_URL || PARTYLOCK_PROD_API_BASE;
}

export function getWsBaseUrl(apiEndpoint: string): string {
  const apiBase = getApiBaseUrl(apiEndpoint);
  return apiBase.replace(/\/api\/v1\/?$/, "");
}
