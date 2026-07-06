import { useState, useEffect, useCallback, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitTo } from "@tauri-apps/api/event";
import { getStore } from "@/lib/tauri";
import { getApiBaseUrl, isLauncherLocalMode, resolveApiEndpoint } from "@/lib/config";

export interface Settings {
  apiEndpoint: string;
  isSettingsReady: boolean;
  setApiEndpoint: (endpoint: string) => void;
  apiUrl: string;
  telemetryEnabled: boolean;
  setTelemetryEnabled: (enabled: boolean) => void;
  hideOwnSkin: boolean;
  setHideOwnSkin: (enabled: boolean) => void;
  hideOthersSkins: boolean;
  setHideOthersSkins: (enabled: boolean) => void;
}

interface SettingsPayload {
  apiEndpoint: string;
  telemetryEnabled: boolean;
  hideOwnSkin: boolean;
  hideOthersSkins: boolean;
}

function toPayload(
  apiEndpoint: string,
  telemetryEnabled: boolean,
  hideOwnSkin: boolean,
  hideOthersSkins: boolean,
): SettingsPayload {
  return { apiEndpoint, telemetryEnabled, hideOwnSkin, hideOthersSkins };
}

export function useSettings(): Settings {
  const devSession = isLauncherLocalMode();
  const [apiEndpoint, setApiEndpointState] = useState(devSession ? "local" : "prod");
  const [telemetryEnabled, setTelemetryEnabledState] = useState(true);
  const [hideOwnSkin, setHideOwnSkinState] = useState(false);
  const [hideOthersSkins, setHideOthersSkinsState] = useState(false);
  const [isSettingsReady, setIsSettingsReady] = useState(false);

  const resolvedEndpoint = useMemo(
    () => resolveApiEndpoint(apiEndpoint),
    [apiEndpoint],
  );

  useEffect(() => {
    getStore()
      .then(async (store) => {
        const storedEndpoint = await store.get<string>("api_endpoint");
        if (devSession) {
          setApiEndpointState("local");
          if (storedEndpoint !== "local") {
            await store.set("api_endpoint", "local");
            await store.save();
          }
        } else if (storedEndpoint) {
          setApiEndpointState(storedEndpoint);
        }

        const telemetry = await store.get<boolean>("telemetry_enabled");
        if (telemetry !== undefined && telemetry !== null) {
          setTelemetryEnabledState(telemetry);
        }
        const ownSkin = await store.get<boolean>("hide_own_skin");
        if (ownSkin !== undefined && ownSkin !== null) {
          setHideOwnSkinState(ownSkin);
        }
        const othersSkins = await store.get<boolean>("hide_others_skins");
        if (othersSkins !== undefined && othersSkins !== null) {
          setHideOthersSkinsState(othersSkins);
        }
      })
      .finally(() => setIsSettingsReady(true));
  }, [devSession]);

  useEffect(() => {
    const unlisten = listen<SettingsPayload>("settings-changed", (event) => {
      const nextEndpoint = devSession ? "local" : event.payload.apiEndpoint;
      setApiEndpointState(nextEndpoint);
      setTelemetryEnabledState(event.payload.telemetryEnabled);
      setHideOwnSkinState(event.payload.hideOwnSkin);
      setHideOthersSkinsState(event.payload.hideOthersSkins);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [devSession]);

  const emit = useCallback((next: SettingsPayload) => {
    emitTo("main", "settings-changed", next);
  }, []);

  const setApiEndpoint = useCallback(async (endpoint: string) => {
    const next = devSession ? "local" : endpoint;
    setApiEndpointState(next);
    const store = await getStore();
    await store.set("api_endpoint", next);
    await store.save();
    emit(toPayload(next, telemetryEnabled, hideOwnSkin, hideOthersSkins));
  }, [devSession, emit, telemetryEnabled, hideOwnSkin, hideOthersSkins]);

  const setTelemetryEnabled = useCallback(async (enabled: boolean) => {
    setTelemetryEnabledState(enabled);
    const store = await getStore();
    await store.set("telemetry_enabled", enabled);
    await store.save();
    emit(toPayload(apiEndpoint, enabled, hideOwnSkin, hideOthersSkins));
  }, [emit, apiEndpoint, hideOwnSkin, hideOthersSkins]);

  const setHideOwnSkin = useCallback(async (enabled: boolean) => {
    setHideOwnSkinState(enabled);
    const store = await getStore();
    await store.set("hide_own_skin", enabled);
    await store.save();
    emit(toPayload(apiEndpoint, telemetryEnabled, enabled, hideOthersSkins));
  }, [emit, apiEndpoint, telemetryEnabled, hideOthersSkins]);

  const setHideOthersSkins = useCallback(async (enabled: boolean) => {
    setHideOthersSkinsState(enabled);
    const store = await getStore();
    await store.set("hide_others_skins", enabled);
    await store.save();
    emit(toPayload(apiEndpoint, telemetryEnabled, hideOwnSkin, enabled));
  }, [emit, apiEndpoint, telemetryEnabled, hideOwnSkin]);

  return {
    apiEndpoint: resolvedEndpoint,
    isSettingsReady,
    setApiEndpoint,
    apiUrl: getApiBaseUrl(apiEndpoint),
    telemetryEnabled,
    setTelemetryEnabled,
    hideOwnSkin,
    setHideOwnSkin,
    hideOthersSkins,
    setHideOthersSkins,
  };
}
