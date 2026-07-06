import { getApiBaseUrl } from "./config";

export interface AuthUser {
  id: string;
  email: string;
  name: string;
  steamId?: string;
  avatarUrl?: string;
  role: string;
  verified: boolean;
}

export function getSteamLauncherAuthUrl(
  apiEndpoint: string,
  callbackPort: number,
): string {
  return `${getApiBaseUrl(apiEndpoint)}/auth/steam/launcher?callback_port=${callbackPort}`;
}

export async function fetchProfile(
  apiEndpoint: string,
  accessToken: string,
): Promise<AuthUser> {
  const res = await fetch(`${getApiBaseUrl(apiEndpoint)}/users/profile`, {
    headers: { Authorization: `Bearer ${accessToken}` },
  });
  if (!res.ok) {
    throw new Error(`profile_http_${res.status}`);
  }
  return (await res.json()) as AuthUser;
}

export function steamAuthErrorMessage(code: string): string {
  if (code === "vac_ban") {
    return "Contas com banimento VAC não podem usar a PartyLock.";
  }
  return "Falha na autenticação via Steam.";
}
