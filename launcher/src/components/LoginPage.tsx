import { useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { getSteamLauncherAuthUrl } from "@/lib/api";
import styles from "./LoginPage.module.css";

interface LoginPageProps {
  apiEndpoint: string;
  isLoading?: boolean;
  isSteamPending?: boolean;
  onSteamStart?: () => void;
}

export default function LoginPage({
  apiEndpoint,
  isLoading = false,
  isSteamPending = false,
  onSteamStart,
}: LoginPageProps) {
  const [error, setError] = useState<string | null>(null);

  async function handleSteamLogin() {
    setError(null);
    try {
      await openUrl(getSteamLauncherAuthUrl(apiEndpoint));
      onSteamStart?.();
    } catch {
      setError("Não foi possível abrir o navegador para login Steam.");
    }
  }

  const busy = isLoading || isSteamPending;

  return (
    <div className={styles.page}>
      <div className={styles.brand}>
        <div className={styles.logo}>PARTYLOCK</div>
        <p className={styles.tagline}>Launcher — entre com Steam para receber partidas</p>
      </div>

      <div className={styles.actions}>
        {error && <p className={styles.error}>{error}</p>}

        <button
          type="button"
          className={styles.steamBtn}
          onClick={handleSteamLogin}
          disabled={busy}
        >
          {isSteamPending ? "AGUARDANDO STEAM…" : "ENTRAR COM STEAM"}
        </button>

        {isSteamPending && (
          <p className={styles.hint}>
            Complete o login no navegador. O launcher abrirá automaticamente.
          </p>
        )}
      </div>
    </div>
  );
}
