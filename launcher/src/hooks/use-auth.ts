import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { fetchProfile, getSteamLauncherAuthUrl, steamAuthErrorMessage } from "@/lib/api";
import { resolveApiEndpoint } from "@/lib/config";
import { getStore } from "@/lib/tauri";
import type { AuthUser } from "@/lib/api";

const TOKEN_KEY = "access_token";
const USER_KEY = "user";
const STEAM_PENDING_TIMEOUT_MS = 3 * 60 * 1000;

interface AuthCallbackPayload {
  access_token?: string;
  error?: string;
}

export interface AuthState {
  user: AuthUser | null;
  accessToken: string | null;
  isLoading: boolean;
  isSteamPending: boolean;
  beginSteamLogin: (apiEndpoint: string) => Promise<void>;
  cancelSteamLogin: () => void;
  logout: () => Promise<void>;
}

function steamCompleteErrorMessage(err: unknown): string {
  if (err instanceof Error && err.message.startsWith("profile_http_")) {
    const status = err.message.slice("profile_http_".length);
    return `Falha ao concluir login via Steam (API respondeu ${status}).`;
  }
  return "Falha ao concluir login via Steam.";
}

export function useAuth(apiEndpoint: string): AuthState {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSteamPending, setIsSteamPending] = useState(false);
  const steamApiEndpointRef = useRef(apiEndpoint);

  const persistSession = useCallback(async (token: string, profile: AuthUser) => {
    const store = await getStore();
    await store.set(TOKEN_KEY, token);
    await store.set(USER_KEY, profile);
    await store.save();
    setAccessToken(token);
    setUser(profile);
  }, []);

  const completeSteamAuth = useCallback(
    async (token: string) => {
      const profile = await fetchProfile(
        resolveApiEndpoint(steamApiEndpointRef.current),
        token,
      );
      await persistSession(token, profile);
      setIsSteamPending(false);
      await invoke("stop_steam_auth_listener");
    },
    [persistSession],
  );

  useEffect(() => {
    getStore()
      .then(async (store) => {
        const token = await store.get<string>(TOKEN_KEY);
        const savedUser = await store.get<AuthUser>(USER_KEY);
        if (!token) return;
        setAccessToken(token);
        if (savedUser) setUser(savedUser);
        try {
          const profile = await fetchProfile(resolveApiEndpoint(apiEndpoint), token);
          await persistSession(token, profile);
        } catch {
          await store.delete(TOKEN_KEY);
          await store.delete(USER_KEY);
          await store.save();
          setAccessToken(null);
          setUser(null);
        }
      })
      .finally(() => setIsLoading(false));
  }, [apiEndpoint, persistSession]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function setup() {
      unlisten = await listen<AuthCallbackPayload>("auth-callback", (event) => {
        const payload = event.payload;
        if (payload.error) {
          setIsSteamPending(false);
          void invoke("stop_steam_auth_listener");
          window.alert(steamAuthErrorMessage(payload.error));
          return;
        }
        if (payload.access_token) {
          completeSteamAuth(payload.access_token).catch((err) => {
            setIsSteamPending(false);
            window.alert(steamCompleteErrorMessage(err));
          });
          return;
        }
        setIsSteamPending(false);
      });

      await invoke("auth_callback_ready");
    }

    setup();

    return () => {
      unlisten?.();
    };
  }, [completeSteamAuth]);

  useEffect(() => {
    if (!isSteamPending) return;
    const timer = window.setTimeout(() => {
      setIsSteamPending(false);
      void invoke("stop_steam_auth_listener");
      window.alert("Tempo esgotado aguardando o Steam.");
    }, STEAM_PENDING_TIMEOUT_MS);
    return () => window.clearTimeout(timer);
  }, [isSteamPending]);

  const logout = useCallback(async () => {
    const store = await getStore();
    await store.delete(TOKEN_KEY);
    await store.delete(USER_KEY);
    await store.save();
    setAccessToken(null);
    setUser(null);
    setIsSteamPending(false);
    await invoke("stop_steam_auth_listener");
  }, []);

  const beginSteamLogin = useCallback(async (endpoint: string) => {
    const resolved = resolveApiEndpoint(endpoint);
    steamApiEndpointRef.current = resolved;
    setIsSteamPending(true);
    try {
      await invoke("stop_steam_auth_listener");
      const port = await invoke<number>("start_steam_auth_listener");
      const authUrl = getSteamLauncherAuthUrl(resolved, port);
      await openUrl(authUrl);
    } catch {
      setIsSteamPending(false);
      await invoke("stop_steam_auth_listener");
      window.alert("Não foi possível iniciar o login Steam.");
    }
  }, []);

  const cancelSteamLogin = useCallback(() => {
    setIsSteamPending(false);
    void invoke("stop_steam_auth_listener");
  }, []);

  return {
    user,
    accessToken,
    isLoading,
    isSteamPending,
    beginSteamLogin,
    cancelSteamLogin,
    logout,
  };
}
