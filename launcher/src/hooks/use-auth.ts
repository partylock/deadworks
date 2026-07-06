import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { fetchProfile, steamAuthErrorMessage } from "@/lib/api";
import { getStore } from "@/lib/tauri";
import type { AuthUser } from "@/lib/api";

const TOKEN_KEY = "access_token";
const USER_KEY = "user";

interface AuthCallbackPayload {
  access_token?: string;
  error?: string;
}

export interface AuthState {
  user: AuthUser | null;
  accessToken: string | null;
  isLoading: boolean;
  isSteamPending: boolean;
  startSteamLogin: () => void;
  logout: () => Promise<void>;
}

export function useAuth(apiEndpoint: string): AuthState {
  const [user, setUser] = useState<AuthUser | null>(null);
  const [accessToken, setAccessToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSteamPending, setIsSteamPending] = useState(false);

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
      const profile = await fetchProfile(apiEndpoint, token);
      await persistSession(token, profile);
      setIsSteamPending(false);
    },
    [apiEndpoint, persistSession],
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
          const profile = await fetchProfile(apiEndpoint, token);
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
          window.alert(steamAuthErrorMessage(payload.error));
          return;
        }
        if (payload.access_token) {
          completeSteamAuth(payload.access_token).catch(() => {
            setIsSteamPending(false);
            window.alert("Falha ao concluir login via Steam.");
          });
        }
      });

      await invoke("auth_callback_ready");
    }

    setup();

    return () => {
      unlisten?.();
    };
  }, [completeSteamAuth]);

  const logout = useCallback(async () => {
    const store = await getStore();
    await store.delete(TOKEN_KEY);
    await store.delete(USER_KEY);
    await store.save();
    setAccessToken(null);
    setUser(null);
    setIsSteamPending(false);
  }, []);

  const startSteamLogin = useCallback(() => setIsSteamPending(true), []);

  return {
    user,
    accessToken,
    isLoading,
    isSteamPending,
    startSteamLogin,
    logout,
  };
}
