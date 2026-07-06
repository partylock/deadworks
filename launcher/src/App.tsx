import { useEffect, useState } from "react";
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

  useEffect(() => {
    if (!socket.matchReady) {
      setConnectTarget(null);
    }
  }, [socket.matchReady]);

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
