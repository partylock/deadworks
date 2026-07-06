import { useState, useEffect, useRef } from "react";
import { prepareAndConnectMatch, listenDownloadProgress } from "@/lib/tauri";
import { resolveMatchConnectAddr } from "@/lib/match-connect";
import type { DownloadProgress, MatchReadyPayload } from "@/lib/types";
import styles from "./ConnectDialog.module.css";
import { cn } from "@/lib/utils";

interface ConnectDialogProps {
  match: MatchReadyPayload;
  onClose: () => void;
}

function overallProgress(p: DownloadProgress): number {
  const itemPct =
    p.total_bytes > 0 ? Math.min(100, Math.round((p.bytes_downloaded / p.total_bytes) * 100)) : 0;
  if (p.total_items <= 0) {
    return itemPct;
  }
  const completed = p.status === "ready" ? p.item_index + 1 : p.item_index;
  const inItem = p.status === "ready" ? 0 : itemPct / 100;
  return Math.min(100, Math.round(((completed + inItem) / p.total_items) * 100));
}

function formatDownloadStatus(p: DownloadProgress): { text: string; progress: number | null } {
  const overall = overallProgress(p);

  switch (p.status) {
    case "fetching":
      return { text: "Preparando conteúdo da partida…", progress: null };
    case "checking":
      return { text: "Verificando arquivos…", progress: overall };
    case "downloading":
      return {
        text: p.total_items > 1 ? `Baixando conteúdo… ${overall}%` : `Baixando… ${overall}%`,
        progress: overall,
      };
    case "decompressing":
      return { text: "Preparando arquivos…", progress: overall };
    case "ready":
      return { text: "Preparando conteúdo…", progress: overall };
    case "connecting":
      return { text: "Conectando via Steam…", progress: 100 };
    default:
      return { text: "Aguarde…", progress: null };
  }
}

export default function ConnectDialog({ match, onClose }: ConnectDialogProps) {
  const [status, setStatus] = useState("Inicializando…");
  const [progress, setProgress] = useState<number | null>(null);
  const startedRef = useRef(false);
  const addr = resolveMatchConnectAddr(match);

  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;

    let unlisten: (() => void) | null = null;

    async function run() {
      unlisten = await listenDownloadProgress((p) => {
        const next = formatDownloadStatus(p);
        setStatus(next.text);
        setProgress(next.progress);
      });

      try {
        setStatus("Preparando conteúdo da partida…");
        const result = await prepareAndConnectMatch(match.matchId, addr);
        if (result.success) {
          setStatus(result.message);
          setProgress(100);
          setTimeout(onClose, 2000);
        } else {
          setStatus(`Erro: ${result.message}`);
        }
      } catch (e) {
        const msg = typeof e === "string" ? e : String(e);
        if (msg.includes("FILE_IN_USE")) {
          setStatus(
            "Um arquivo desta partida está em uso no Deadlock. Feche o jogo e tente novamente.",
          );
        } else {
          setStatus(`Erro: ${msg}`);
        }
      }
    }

    run();

    return () => {
      unlisten?.();
    };
  }, [match.matchId, addr, onClose]);

  return (
    <div className={styles.overlay} onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className={styles.box}>
        <h3 className={styles.title}>Conectando à partida</h3>
        <p className={styles.serverName}>PartyLock</p>

        <div className={styles.progressTrack}>
          <div
            className={cn(styles.progressBar, progress == null && styles.progressIndeterminate)}
            style={progress != null ? { width: `${progress}%` } : undefined}
          />
        </div>

        <p className={styles.status}>{status}</p>
        <button onClick={onClose} className={styles.cancelBtn}>
          CANCELAR
        </button>
      </div>
    </div>
  );
}
