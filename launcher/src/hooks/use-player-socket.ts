import { useCallback, useEffect, useRef, useState } from "react";
import { io, Socket } from "socket.io-client";
import { getWsBaseUrl } from "@/lib/config";
import { fetchLauncherState } from "@/lib/launcher-state";
import type {
  MatchConnectStatusPayload,
  MatchFailedNotification,
  MatchProvisioningPayload,
  MatchReadyPayload,
} from "@/lib/types";

export interface PlayerSocketState {
  connected: boolean;
  matchProvisioning: MatchProvisioningPayload | null;
  matchReady: MatchReadyPayload | null;
  matchConnectStatus: MatchConnectStatusPayload | null;
  matchFailed: MatchFailedNotification | null;
  error: string | null;
  dismissMatchFailed: () => void;
}

export function usePlayerSocket(
  apiEndpoint: string,
  accessToken: string | null,
): PlayerSocketState {
  const [connected, setConnected] = useState(false);
  const [matchProvisioning, setMatchProvisioning] =
    useState<MatchProvisioningPayload | null>(null);
  const [matchReady, setMatchReady] = useState<MatchReadyPayload | null>(null);
  const [matchConnectStatus, setMatchConnectStatus] =
    useState<MatchConnectStatusPayload | null>(null);
  const [matchFailed, setMatchFailed] = useState<MatchFailedNotification | null>(null);
  const [error, setError] = useState<string | null>(null);
  const socketRef = useRef<Socket | null>(null);
  const accessTokenRef = useRef(accessToken);
  const dismissedFailedMatchIdsRef = useRef<Set<string>>(new Set());
  accessTokenRef.current = accessToken;

  const showMatchFailed = useCallback((payload: MatchFailedNotification) => {
    if (dismissedFailedMatchIdsRef.current.has(payload.matchId)) {
      return;
    }
    setMatchFailed(payload);
    setMatchReady(null);
    setMatchProvisioning(null);
    setMatchConnectStatus(null);
  }, []);

  const syncFromApi = async () => {
    const token = accessTokenRef.current;
    if (!token) return;
    try {
      const state = await fetchLauncherState(apiEndpoint, token);
      setMatchProvisioning(state.matchProvisioning);
      setMatchReady(state.matchReady);
      setMatchConnectStatus(state.matchConnectStatus);
    } catch {
      // ponytail: REST sync is best-effort; WS remains primary
    }
  };

  useEffect(() => {
    if (!accessToken) {
      setConnected(false);
      setMatchProvisioning(null);
      setMatchReady(null);
      setMatchConnectStatus(null);
      setMatchFailed(null);
      setError(null);
      dismissedFailedMatchIdsRef.current.clear();
      return;
    }

    const socket = io(`${getWsBaseUrl(apiEndpoint)}/draft`, {
      auth: { token: accessToken },
      transports: ["websocket", "polling"],
    });
    socketRef.current = socket;

    socket.on("connect", () => setConnected(false));
    socket.on("authenticated", () => {
      setConnected(true);
      setError(null);
      void syncFromApi();
    });
    socket.on("disconnect", () => setConnected(false));
    socket.on("error", (payload: { message?: string }) => {
      setError(payload?.message ?? "Connection error");
    });

    socket.on("match-provisioning", (payload: MatchProvisioningPayload) => {
      setMatchProvisioning(payload);
      setMatchFailed(null);
    });
    socket.on("match-ready", (payload: MatchReadyPayload) => {
      setMatchReady(payload);
      setMatchFailed(null);
    });
    socket.on(
      "match-connect-status",
      (payload: MatchConnectStatusPayload | null) => {
        setMatchConnectStatus(payload);
      },
    );
    socket.on("match-failed", (payload: MatchFailedNotification) => {
      showMatchFailed(payload);
    });
    socket.on("match-ended", () => {
      setMatchReady(null);
      setMatchProvisioning(null);
      setMatchConnectStatus(null);
    });

    const pollId = window.setInterval(() => {
      void syncFromApi();
    }, 4000);

    return () => {
      window.clearInterval(pollId);
      socket.disconnect();
      socketRef.current = null;
    };
  }, [apiEndpoint, accessToken, showMatchFailed]);

  const dismissMatchFailed = useCallback(() => {
    setMatchFailed((current) => {
      if (current?.matchId) {
        dismissedFailedMatchIdsRef.current.add(current.matchId);
      }
      return null;
    });
  }, []);

  return {
    connected,
    matchProvisioning,
    matchReady,
    matchConnectStatus,
    matchFailed,
    error,
    dismissMatchFailed,
  };
}
