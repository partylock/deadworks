import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Server } from "@/lib/types";

type DeepLinkPayload =
  | { kind: "id"; value: string }
  | { kind: "ip"; value: string }
  | { kind: "error"; value: string };

export interface DeepLinkRequest {
  requestId: number;
  server: Server | null;
  error: string | null;
}

export function useDeepLink(apiUrl: string) {
  const [request, setRequest] = useState<DeepLinkRequest | null>(null);
  const counterRef = useRef(0);
  const apiUrlRef = useRef(apiUrl);

  useEffect(() => {
    apiUrlRef.current = apiUrl;
  }, [apiUrl]);

  const resolvePayload = useCallback(async (p: DeepLinkPayload) => {
    const requestId = ++counterRef.current;
    if (p.kind === "error") {
      setRequest({ requestId, server: null, error: p.value });
      return;
    }
    const base = apiUrlRef.current;
    const url =
      p.kind === "id"
        ? `${base}/api/servers/${encodeURIComponent(p.value)}`
        : `${base}/api/servers/lookup?address=${encodeURIComponent(p.value)}`;
    try {
      const res = await fetch(url, { cache: "no-store" });
      if (res.status === 404) {
        const msg =
          p.kind === "ip"
            ? `No online server found at ${p.value}`
            : `Server ${p.value} not found`;
        setRequest({ requestId, server: null, error: msg });
        return;
      }
      if (!res.ok) {
        setRequest({
          requestId,
          server: null,
          error: `Lookup failed (HTTP ${res.status})`,
        });
        return;
      }
      const server = (await res.json()) as Server;
      setRequest({ requestId, server, error: null });
    } catch (e) {
      setRequest({ requestId, server: null, error: String(e) });
    }
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<DeepLinkPayload>("deep-link://connect", (evt) => {
        resolvePayload(evt.payload);
      });
      // Idempotent: backend sets ready=true and replays any pending URLs via
      // the same event we just subscribed to.
      await invoke("deep_link_ready");
    })();

    return () => {
      unlisten?.();
    };
  }, [resolvePayload]);

  const clear = useCallback(() => setRequest(null), []);
  return { request, clear };
}
