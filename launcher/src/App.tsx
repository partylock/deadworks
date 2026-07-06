import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import Titlebar from "@/components/Titlebar";
import LoginPage from "@/components/LoginPage";
import MatchPanel from "@/components/MatchPanel";
import ConnectDialog from "@/components/ConnectDialog";
import { useSettings } from "@/hooks/use-settings";
import { useAuth } from "@/hooks/use-auth";
import { usePlayerSocket } from "@/hooks/use-player-socket";
import type { MatchReadyPayload } from "@/lib/types";
import styles from "./App.module.css";

export default function App() {
  const { apiEndpoint } = useSettings();
  const { user, accessToken, isLoading, isSteamPending, startSteamLogin, logout } =
    useAuth(apiEndpoint);
  const socket = usePlayerSocket(apiEndpoint, accessToken);
  const [connectTarget, setConnectTarget] = useState<MatchReadyPayload | null>(null);
  const [pendingDeepLinkMatch, setPendingDeepLinkMatch] = useState<MatchReadyPayload | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<{
        match_id: string;
        host: string;
        port: number;
        local_host?: string;
      }>("match-connect", (event) => {
        const payload = event.payload;
        const match: MatchReadyPayload = {
          matchId: payload.match_id,
          draftRoomId: "",
          host: payload.host,
          port: payload.port,
          connectCommand: `connect ${payload.host}:${payload.port}`,
          localHost: payload.local_host,
          localConnectCommand: payload.local_host
            ? `connect ${payload.local_host}:${payload.port}`
            : undefined,
        };
        setPendingDeepLinkMatch(match);
      });
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!socket.matchReady) {
      setConnectTarget(null);
    }
  }, [socket.matchReady]);

  useEffect(() => {
    if (!user || !accessToken || !pendingDeepLinkMatch) return;
    setConnectTarget(pendingDeepLinkMatch);
    setPendingDeepLinkMatch(null);
  }, [user, accessToken, pendingDeepLinkMatch]);

  return (
    <>
      <Titlebar />
      <main className={styles.main}>
        {isLoading ? (
          <div className={styles.loading}>Carregando…</div>
        ) : !user || !accessToken ? (
          <LoginPage
            apiEndpoint={apiEndpoint}
            isLoading={isLoading}
            isSteamPending={isSteamPending}
            onSteamStart={startSteamLogin}
          />
        ) : (
          <MatchPanel
            user={user}
            connected={socket.connected}
            socketError={socket.error}
            matchProvisioning={socket.matchProvisioning}
            matchReady={socket.matchReady}
            matchConnectStatus={socket.matchConnectStatus}
            matchFailed={socket.matchFailed}
            onDismissMatchFailed={socket.dismissMatchFailed}
            onConnect={() => socket.matchReady && setConnectTarget(socket.matchReady)}
            onLogout={logout}
          />
        )}
      </main>
      {connectTarget && (
        <ConnectDialog
          key={connectTarget.matchId}
          match={connectTarget}
          onClose={() => setConnectTarget(null)}
        />
      )}
    </>
  );
}
