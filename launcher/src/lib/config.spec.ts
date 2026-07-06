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

assert.equal(
  getApiBaseUrl("prod"),
  "https://api.partylock.com.br/api/v1",
);

assert.equal(
  getApiBaseUrl("local"),
  "http://localhost:3001/api/v1",
);

assert.equal(isLauncherLocalMode(), false);
