import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { emitTo } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useSettings } from "@/hooks/use-settings";
import { getStore } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import styles from "./SettingsWindow.module.css";

const NAV_ITEMS = [
  { id: "general", label: "Geral" },
  ...(import.meta.env.DEV ? [{ id: "developer", label: "Desenvolvedor" }] : []),
];

function SettingRow({
  title,
  description,
  control,
}: {
  title: string;
  description: string;
  control: React.ReactNode;
}) {
  return (
    <div className={styles.settingRow}>
      <div className={styles.settingInfo}>
        <div className={styles.settingLabel}>{title}</div>
        <div className={styles.settingDesc}>{description}</div>
      </div>
      <div>{control}</div>
    </div>
  );
}

export default function SettingsWindow() {
  const {
    apiEndpoint,
    setApiEndpoint,
    telemetryEnabled,
    setTelemetryEnabled,
    hideOwnSkin,
    setHideOwnSkin,
    hideOthersSkins,
    setHideOthersSkins,
  } = useSettings();
  const [activeSection, setActiveSection] = useState("general");
  const [autostart, setAutostart] = useState(false);
  const [detectedPath, setDetectedPath] = useState<string | null>(null);
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [isOverridden, setIsOverridden] = useState(false);
  const [gameDirError, setGameDirError] = useState<string | null>(null);
  const win = getCurrentWindow();

  useEffect(() => {
    invoke<boolean>("plugin:autostart|is_enabled").then(setAutostart).catch(() => {});
  }, []);

  useEffect(() => {
    invoke<string>("get_detected_game_dir")
      .then(setDetectedPath)
      .catch(() => setDetectedPath(null));

    getStore().then(async (store) => {
      const override = await store.get<string>("game_dir_override");
      if (override) {
        setCurrentPath(override);
        setIsOverridden(true);
      } else {
        invoke<string>("get_detected_game_dir")
          .then(setCurrentPath)
          .catch(() => {});
        setIsOverridden(false);
      }
    });
  }, []);

  const handleBrowseGameDir = async () => {
    const selected = await open({ directory: true, title: "Selecionar pasta do Deadlock" });
    if (!selected) return;
    try {
      await invoke("set_game_dir", { path: selected });
      const store = await getStore();
      await store.set("game_dir_override", selected);
      await store.save();
      setCurrentPath(selected);
      setIsOverridden(true);
      setGameDirError(null);
    } catch (e) {
      setGameDirError(typeof e === "string" ? e : String(e));
    }
  };

  const handleResetGameDir = async () => {
    invoke("reset_game_dir");
    const store = await getStore();
    await store.delete("game_dir_override");
    await store.save();
    setCurrentPath(detectedPath);
    setIsOverridden(false);
    setGameDirError(null);
  };

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await invoke(enabled ? "plugin:autostart|enable" : "plugin:autostart|disable");
      setAutostart(enabled);
      const store = await getStore();
      await store.set("autostart_set", true);
      await store.save();
    } catch (e) {
      console.error("Failed to toggle autostart:", e);
    }
  };

  return (
    <div className={styles.window}>
      <div className={styles.titlebar} data-tauri-drag-region>
        <div className={styles.titlebarLeft}>
          <span className={styles.titlebarTitle}>CONFIGURAÇÕES</span>
        </div>
        <div className={styles.windowControls}>
          <button onClick={() => win.minimize()} className={styles.winBtn} aria-label="Minimizar">
            <svg width="10" height="1" viewBox="0 0 10 1">
              <rect width="10" height="1" fill="currentColor" />
            </svg>
          </button>
          <button onClick={() => win.toggleMaximize()} className={styles.winBtn} aria-label="Maximizar">
            <svg width="10" height="10" viewBox="0 0 10 10">
              <rect width="10" height="10" fill="none" stroke="currentColor" strokeWidth="1" />
            </svg>
          </button>
          <button onClick={() => win.close()} className={cn(styles.winBtn, styles.winClose)} aria-label="Fechar">
            <svg width="10" height="10" viewBox="0 0 10 10">
              <line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" strokeWidth="1.2" />
              <line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" strokeWidth="1.2" />
            </svg>
          </button>
        </div>
      </div>

      <div className={styles.body}>
        <div className={styles.sidebar}>
          <div className={styles.navList}>
            {NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                onClick={() => setActiveSection(item.id)}
                className={cn(
                  styles.navItem,
                  activeSection === item.id && styles.navItemActive
                )}
              >
                {item.label}
              </button>
            ))}
          </div>
        </div>

        <div className={styles.content}>
          {activeSection === "general" && (
            <>
              <h2 className={styles.sectionTitle}>Geral</h2>
              <div className={styles.sectionSubtitle}>Inicialização</div>

              <SettingRow
                title="Iniciar com o Windows"
                description="Abrir o launcher PartyLock ao fazer login"
                control={
                  <button
                    className={cn(styles.toggle, autostart && styles.toggleOn)}
                    onClick={() => toggleAutostart(!autostart)}
                    role="switch"
                    aria-checked={autostart}
                  >
                    <span className={styles.toggleThumb} />
                  </button>
                }
              />

              <div className={styles.sectionSubtitle}>Pasta do jogo</div>

              <div className={styles.settingRow}>
                <div className={styles.settingInfo}>
                  <div className={styles.settingLabel}>Pasta de instalação do Deadlock</div>
                  <div className={styles.settingDesc}>
                    {isOverridden ? "Definida manualmente" : "Detectada automaticamente"}
                    {isOverridden && detectedPath && (
                      <> — detectada: <span className={styles.pathMono}>{detectedPath}</span></>
                    )}
                  </div>
                  <div className={styles.pathDisplay}>
                    {currentPath || "Não encontrada — defina manualmente"}
                  </div>
                  {gameDirError && (
                    <div className={styles.pathError}>{gameDirError}</div>
                  )}
                </div>
                <div className={styles.pathButtons}>
                  <button className={styles.devBtn} onClick={handleBrowseGameDir}>
                    Procurar
                  </button>
                  {isOverridden && (
                    <button className={styles.devBtn} onClick={handleResetGameDir}>
                      Restaurar
                    </button>
                  )}
                </div>
              </div>

              <div className={styles.sectionSubtitle}>Atualizações</div>

              <SettingRow
                title="Verificar atualizações"
                description="Verifica se há uma nova versão do launcher disponível"
                control={
                  <button
                    className={styles.devBtn}
                    onClick={() => emitTo("main", "check-for-updates")}
                  >
                    Verificar agora
                  </button>
                }
              />

              <div className={styles.sectionSubtitle}>Skins</div>

              <SettingRow
                title="Ocultar minha skin equipada"
                description="Só a sua — quando ativado, sua skin equipada não é baixada nem carregada nas partidas."
                control={
                  <button
                    className={cn(styles.toggle, hideOwnSkin && styles.toggleOn)}
                    onClick={() => setHideOwnSkin(!hideOwnSkin)}
                    role="switch"
                    aria-checked={hideOwnSkin}
                  >
                    <span className={styles.toggleThumb} />
                  </button>
                }
              />

              <SettingRow
                title="Ocultar skins dos outros"
                description="Quando ativado, as skins dos outros jogadores não são baixadas — você só vê o visual padrão."
                control={
                  <button
                    className={cn(styles.toggle, hideOthersSkins && styles.toggleOn)}
                    onClick={() => setHideOthersSkins(!hideOthersSkins)}
                    role="switch"
                    aria-checked={hideOthersSkins}
                  >
                    <span className={styles.toggleThumb} />
                  </button>
                }
              />

              <div className={styles.sectionSubtitle}>Privacidade</div>

              <SettingRow
                title="Compartilhar uso anônimo"
                description="Envia um ID aleatório, versão do app e sistema operacional para medir instalações e usuários ativos. Sem dados pessoais. Desative para não enviar."
                control={
                  <button
                    className={cn(styles.toggle, telemetryEnabled && styles.toggleOn)}
                    onClick={() => setTelemetryEnabled(!telemetryEnabled)}
                    role="switch"
                    aria-checked={telemetryEnabled}
                  >
                    <span className={styles.toggleThumb} />
                  </button>
                }
              />
            </>
          )}

          {activeSection === "developer" && (
            <>
              <h2 className={styles.sectionTitle}>Desenvolvedor</h2>
              <div className={styles.sectionSubtitle}>Ferramentas</div>

              <SettingRow
                title="Endpoint da API"
                description="Backend PartyLock"
                control={
                  <select
                    value={apiEndpoint}
                    onChange={(e) => setApiEndpoint(e.target.value)}
                    className={styles.select}
                  >
                    <option value="prod">Produção</option>
                    <option value="local">Local (localhost:3001)</option>
                  </select>
                }
              />

              <SettingRow
                title="Testar UI de atualização"
                description="Simula uma atualização disponível para visualizar o gerenciador"
                control={
                  <button
                    className={styles.devBtn}
                    onClick={() => emitTo("main", "test-update-ui")}
                  >
                    Disparar
                  </button>
                }
              />
            </>
          )}
        </div>
      </div>
    </div>
  );
}
