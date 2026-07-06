import { getApiBaseUrl } from "./config";
import type {
  MatchConnectStatusPayload,
  MatchProvisioningPayload,
  MatchReadyPayload,
} from "./types";

export interface LauncherState {
  matchProvisioning: MatchProvisioningPayload | null;
  matchReady: MatchReadyPayload | null;
  matchConnectStatus: MatchConnectStatusPayload | null;
}

export async function fetchLauncherState(
  apiEndpoint: string,
  accessToken: string,
): Promise<LauncherState> {
  const res = await fetch(`${getApiBaseUrl(apiEndpoint)}/draft/launcher-state`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  if (!res.ok) {
    throw new Error("Failed to sync launcher state");
  }
  return res.json() as Promise<LauncherState>;
}
