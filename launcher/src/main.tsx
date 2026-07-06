import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import SettingsWindow from "./components/SettingsWindow";
import { setLauncherDebugBuild } from "./lib/config";
import "./index.css";

const path = window.location.pathname;

function Root() {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    invoke<boolean>("is_debug_build")
      .then((isDebug) => setLauncherDebugBuild(isDebug))
      .catch(() => setLauncherDebugBuild(false))
      .finally(() => setReady(true));
  }, []);

  if (!ready) {
    return <div style={{ padding: 24, color: "#aaa" }}>Carregando…</div>;
  }

  if (path === "/settings") return <SettingsWindow />;
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
