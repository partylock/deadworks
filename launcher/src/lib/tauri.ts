import { getCurrentWindow } from "@tauri-apps/api/window";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { load, type Store } from "@tauri-apps/plugin-store";
import type { ConnectResult, DownloadProgress } from "./types";

// ── Window controls ──

const appWindow = getCurrentWindow();

export function minimizeWindow() {
  appWindow.minimize();
}

export async function toggleMaximize() {
  if (await appWindow.isMaximized()) {
    appWindow.unmaximize();
  } else {
    appWindow.maximize();
  }
}

export function closeWindow() {
  appWindow.close();
}

export async function openSettingsWindow() {
  try {
    const existing = await WebviewWindow.getByLabel("settings");
    if (existing !== null) {
      try {
        await existing.unminimize();
        await existing.setFocus();
        return;
      } catch (error) {
        console.error("Error focusing settings window:", error);
      }
    }
  } catch (error) {
    console.error("Error retrieving settings window:", error);
  }

  const webview = new WebviewWindow("settings", {
    url: "/settings",
    title: "Settings",
    width: 900,
    height: 700,
    center: true,
    decorations: false,
    resizable: true,
  });

  webview.once("tauri://error", function (e) {
    console.error("Error opening settings window:", e);
  });
}

// ── Store ──

let storePromise: Promise<Store> | null = null;

export function getStore(): Promise<Store> {
  if (!storePromise) {
    storePromise = load("settings.json");
  }
  return storePromise;
}

export async function getStoredAccessToken(): Promise<string | null> {
  const store = await getStore();
  const token = await store.get<string>("access_token");
  return token ?? null;
}

export function prepareAndConnectMatch(
  matchId: string,
  addr: string,
): Promise<ConnectResult> {
  return invoke<ConnectResult>("prepare_and_connect_match", {
    matchId,
    addr,
  });
}

export function listenDownloadProgress(
  callback: (progress: DownloadProgress) => void
): Promise<UnlistenFn> {
  return listen<DownloadProgress>("download-progress", (event) => {
    callback(event.payload);
  });
}

