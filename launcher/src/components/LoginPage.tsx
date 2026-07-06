import styles from "./LoginPage.module.css";

interface LoginPageProps {
  apiEndpoint: string;
  isLoading?: boolean;
  isSteamPending?: boolean;
  onSteamLogin?: (apiEndpoint: string) => Promise<void>;
  onSteamCancel?: () => void;
}

export default function LoginPage({
  apiEndpoint,
  isLoading = false,
  isSteamPending = false,
  onSteamLogin,
  onSteamCancel,
}: LoginPageProps) {
  const busy = isLoading || isSteamPending;

  return (
    <div className={styles.page}>
      <div className={styles.brand}>
        <img src="/logo.png" alt="PartyLock" className={styles.logo} />
        <p className={styles.tagline}>Launcher — entre com Steam para receber partidas</p>
      </div>

      <div className={styles.actions}>
        <button
          type="button"
          className={styles.steamBtn}
          onClick={() => onSteamLogin?.(apiEndpoint)}
          disabled={busy}
        >
          {isSteamPending ? "AGUARDANDO STEAM…" : "ENTRAR COM STEAM"}
        </button>

        {isSteamPending && onSteamCancel && (
          <button type="button" className={styles.cancelBtn} onClick={onSteamCancel}>
            Cancelar
          </button>
        )}
      </div>
    </div>
  );
}
