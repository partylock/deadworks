import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitTo } from "@tauri-apps/api/event";
import { getStore } from "@/lib/tauri";
import { getApiBaseUrl } from "@/lib/config";

export interface Settings {
  apiEndpoint: string;
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
  const [apiEndpoint, setApiEndpointState] = useState(
    import.meta.env.DEV ? "local" : "prod",
  );
  const [telemetryEnabled, setTelemetryEnabledState] = useState(true);
  const [hideOwnSkin, setHideOwnSkinState] = useState(false);
  const [hideOthersSkins, setHideOthersSkinsState] = useState(false);

  useEffect(() => {
    getStore().then(async (store) => {
      const endpoint = await store.get<string>("api_endpoint");
      if (endpoint) setApiEndpointState(endpoint);
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
    });
  }, []);

  useEffect(() => {
    const unlisten = listen<SettingsPayload>("settings-changed", (event) => {
      setApiEndpointState(event.payload.apiEndpoint);
      setTelemetryEnabledState(event.payload.telemetryEnabled);
      setHideOwnSkinState(event.payload.hideOwnSkin);
      setHideOthersSkinsState(event.payload.hideOthersSkins);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const emit = useCallback((next: SettingsPayload) => {
    emitTo("main", "settings-changed", next);
  }, []);

  const setApiEndpoint = useCallback(async (endpoint: string) => {
    setApiEndpointState(endpoint);
    const store = await getStore();
    await store.set("api_endpoint", endpoint);
    await store.save();
    emit(toPayload(endpoint, telemetryEnabled, hideOwnSkin, hideOthersSkins));
  }, [emit, telemetryEnabled, hideOwnSkin, hideOthersSkins]);

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
    apiEndpoint,
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
