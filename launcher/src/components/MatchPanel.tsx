import { cn } from "@/lib/utils";
import type {
  MatchConnectStatusPayload,
  MatchFailedNotification,
  MatchProvisioningPayload,
  MatchReadyPayload,
} from "@/lib/types";
import type { AuthUser } from "@/lib/api";
import styles from "./MatchPanel.module.css";

function userInitials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return `${parts[0][0] ?? ""}${parts[parts.length - 1][0] ?? ""}`.toUpperCase();
}

interface MatchPanelProps {
  user: AuthUser;
  connected: boolean;
  socketError: string | null;
  matchProvisioning: MatchProvisioningPayload | null;
  matchReady: MatchReadyPayload | null;
  matchConnectStatus: MatchConnectStatusPayload | null;
  matchFailed: MatchFailedNotification | null;
  onConnect: () => void;
  onDismissMatchFailed: () => void;
  onLogout: () => void;
}

export default function MatchPanel({
  user,
  connected,
  socketError,
  matchProvisioning,
  matchReady,
  matchConnectStatus,
  matchFailed,
  onConnect,
  onDismissMatchFailed,
  onLogout,
}: MatchPanelProps) {
  const isProvisioning = Boolean(matchProvisioning && !matchReady);
  const canConnect = Boolean(matchReady);
  const statusLabel = matchReady
    ? "Partida pronta"
    : matchProvisioning
      ? "Preparando partida…"
      : connected
        ? "Online"
        : "Reconectando…";

  return (
    <div className={styles.page}>
      <div className={styles.header}>
        <div className={styles.userBlock}>
          <span className={styles.greeting}>Conectado como</span>
          <div className={styles.userRow}>
            <div className={styles.avatar} aria-hidden>
              {user.avatarUrl ? (
                <img src={user.avatarUrl} alt="" className={styles.avatarImg} />
              ) : (
                <span className={styles.avatarFallback}>{userInitials(user.name)}</span>
              )}
            </div>
            <div className={styles.userInfo}>
              <span className={styles.userName}>{user.name}</span>
              <div className={styles.statusRow}>
                <span className={cn(styles.dot, connected && styles.dotOnline)} />
                {statusLabel}
              </div>
            </div>
          </div>
        </div>
        <button type="button" className={styles.logoutBtn} onClick={onLogout}>
          SAIR
        </button>
      </div>

      {socketError && (
        <div className={cn(styles.card, styles.errorCard)}>
          <p className={styles.errorText}>{socketError}</p>
        </div>
      )}

      {matchFailed && (
        <div className={cn(styles.card, styles.errorCard)}>
          <h2 className={styles.title}>Partida cancelada</h2>
          <p className={styles.errorText}>{matchFailed.reason}</p>
          <button type="button" className={styles.dismissBtn} onClick={onDismissMatchFailed}>
            OK
          </button>
        </div>
      )}

      {!matchFailed && !isProvisioning && !canConnect && (
        <div className={styles.card}>
          <div className={styles.idleIcon}>⏳</div>
          <h2 className={styles.title}>Aguardando partida</h2>
          <p className={styles.subtitle}>
            Faça o draft no site PartyLock. Quando a partida estiver pronta, o botão
            de conectar aparece aqui automaticamente.
          </p>
        </div>
      )}

      {isProvisioning && !canConnect && (
        <div className={styles.card}>
          <h2 className={styles.title}>Preparando servidor</h2>
          <p className={styles.subtitle}>{matchProvisioning?.label}</p>
          <div className={styles.steps}>
            {matchProvisioning?.steps.map((step) => (
              <div key={step.step} className={styles.step}>
                <span>•</span>
                <span>{step.label}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {canConnect && matchReady && (
        <div className={styles.card}>
          <h2 className={styles.title}>Partida pronta</h2>
          {isProvisioning && matchProvisioning?.label && (
            <p className={styles.subtitle}>{matchProvisioning.label}</p>
          )}
          <p className={styles.subtitle}>
            Servidor disponível. Clique para se conectar.
          </p>
          {matchConnectStatus && (
            <p className={styles.connectMeta}>
              {matchConnectStatus.connectedCount}/{matchConnectStatus.rosterCount} conectados
            </p>
          )}
          <button type="button" className={styles.connectBtn} onClick={onConnect}>
            CONECTAR
          </button>
        </div>
      )}
    </div>
  );
}
