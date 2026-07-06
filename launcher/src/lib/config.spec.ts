import assert from "node:assert/strict";
import {
  getApiBaseUrl,
  isLauncherLocalMode,
  resolveApiEndpoint,
  setLauncherDebugBuild,
} from "./config";

assert.equal(resolveApiEndpoint("prod"), "prod");
assert.equal(resolveApiEndpoint("local"), "local");

setLauncherDebugBuild(true);
assert.equal(isLauncherLocalMode(), true);
assert.equal(getApiBaseUrl("prod"), "http://localhost:3001/api/v1");
setLauncherDebugBuild(false);

assert.equal(isLauncherLocalMode(), false);

const g = globalThis as typeof globalThis & {
  window?: { location: { hostname: string; port: string } };
};
const prevWindow = g.window;
g.window = { location: { hostname: "tauri.localhost", port: "" } };
assert.equal(
  isLauncherLocalMode(),
  false,
  "packaged prod webview must not force local API",
);
g.window = prevWindow;

assert.equal(
  getApiBaseUrl("prod"),
  "https://api.partylock.com.br/api/v1",
);

assert.equal(
  getApiBaseUrl("local"),
  "http://localhost:3001/api/v1",
);

assert.equal(isLauncherLocalMode(), false);
