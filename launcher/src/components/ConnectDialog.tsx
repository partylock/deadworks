import { useState, useEffect, useRef } from "react";
import { prepareAndConnectMatch, listenDownloadProgress } from "@/lib/tauri";
import { resolveMatchConnectAddr } from "@/lib/match-connect";
import type { MatchReadyPayload } from "@/lib/types";
import styles from "./ConnectDialog.module.css";
import { cn } from "@/lib/utils";

interface ConnectDialogProps {
  match: MatchReadyPayload;
  onClose: () => void;
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
        if (p.status === "fetching") {
          setStatus("Verificando mods da partida…");
        } else if (p.status === "checking") {
          setStatus(`Verificando ${p.name}… (${p.item_index + 1}/${p.total_items})`);
          setProgress(null);
        } else if (p.status === "downloading") {
          const pct =
            p.total_bytes > 0
              ? Math.round((p.bytes_downloaded / p.total_bytes) * 100)
              : 0;
          setStatus(`Baixando ${p.name}… ${pct}% (${p.item_index + 1}/${p.total_items})`);
          setProgress(pct);
        } else if (p.status === "decompressing") {
          const pct =
            p.total_bytes > 0
              ? Math.min(100, Math.round((p.bytes_downloaded / p.total_bytes) * 100))
              : 0;
          setStatus(`Descompactando ${p.name}… ${pct}% (${p.item_index + 1}/${p.total_items})`);
          setProgress(pct);
        } else if (p.status === "ready") {
          setStatus(`${p.name} ok (${p.item_index + 1}/${p.total_items})`);
          setProgress(100);
        } else if (p.status === "connecting") {
          setStatus("Conectando ao servidor…");
          setProgress(100);
        }
      });

      try {
        setStatus("Verificando mods da partida…");
        const result = await prepareAndConnectMatch(match.matchId, addr);
        if (result.success) {
          setStatus(result.message);
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
        <p className={styles.serverName}>PartyLock Match</p>
        <p className={styles.addr}>{addr}</p>

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
