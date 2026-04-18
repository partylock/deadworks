import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { emitTo } from "@tauri-apps/api/event";
import { getStore, getApiUrl } from "@/lib/tauri";

export interface Settings {
  apiEndpoint: string;
  setApiEndpoint: (endpoint: string) => void;
  apiUrl: string;
  telemetryEnabled: boolean;
  setTelemetryEnabled: (enabled: boolean) => void;
}

interface SettingsPayload {
  apiEndpoint: string;
  telemetryEnabled: boolean;
}

export function useSettings(): Settings {
  const [apiEndpoint, setApiEndpointState] = useState("prod");
  const [telemetryEnabled, setTelemetryEnabledState] = useState(true);

  useEffect(() => {
    getStore().then(async (store) => {
      const endpoint = await store.get<string>("api_endpoint");
      if (endpoint) setApiEndpointState(endpoint);
      const telemetry = await store.get<boolean>("telemetry_enabled");
      if (telemetry !== undefined && telemetry !== null) {
        setTelemetryEnabledState(telemetry);
      }
    });
  }, []);

  useEffect(() => {
    const unlisten = listen<SettingsPayload>("settings-changed", (event) => {
      setApiEndpointState(event.payload.apiEndpoint);
      setTelemetryEnabledState(event.payload.telemetryEnabled);
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
    emit({ apiEndpoint: endpoint, telemetryEnabled });
  }, [emit, telemetryEnabled]);

  const setTelemetryEnabled = useCallback(async (enabled: boolean) => {
    setTelemetryEnabledState(enabled);
    const store = await getStore();
    await store.set("telemetry_enabled", enabled);
    await store.save();
    emit({ apiEndpoint, telemetryEnabled: enabled });
  }, [emit, apiEndpoint]);

  return {
    apiEndpoint,
    setApiEndpoint,
    apiUrl: getApiUrl(apiEndpoint),
    telemetryEnabled,
    setTelemetryEnabled,
  };
}
