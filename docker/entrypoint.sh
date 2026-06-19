#!/bin/bash
set -euo pipefail

# Deadworks Linux entrypoint (based on raimannma/deadworks) with PartyLock match hooks.
# Per-match plugin config is bind-mounted at /match-instance/configs by the orchestrator.

APP_ID=1422450
STEAM_LOGIN="${STEAM_LOGIN:-}"
STEAM_PASSWORD="${STEAM_PASSWORD:-}"
SERVER_PORT="${SERVER_PORT:-27015}"
SERVER_MAP="${SERVER_MAP:-dl_midtown}"
SERVER_PASSWORD="${SERVER_PASSWORD:-}"
RCON_PASSWORD="${RCON_PASSWORD:-}"
PROTON_VERSION="${PROTON_VERSION:-GE-Proton10-33}"
DOTNET_VERSION="${DOTNET_VERSION:-10.0.0}"
DEADWORKS_ARGS="${DEADWORKS_ARGS:-}"
MATCH_PUBLIC_HOST="${MATCH_PUBLIC_HOST:-}"

INSTALL_DIR="/home/steam/server"
PROTON_DIR="/opt/proton"
STEAM_PATH="/home/steam/.steam/steam"
COMPAT_DATA="${STEAM_PATH}/steamapps/compatdata/${APP_ID}"
PFXDIR="${COMPAT_DATA}/pfx"
WIN64_DIR="${INSTALL_DIR}/game/bin/win64"
REDIST_DIR="${STEAM_PATH}/steamapps/common/Steamworks SDK Redist"
GAME_BASE_DIR="/opt/gamefiles"

if [ ! -s /etc/machine-id ]; then
 echo "[phase 0] Generating unique machine-id for this instance..."
 cat /proc/sys/kernel/random/uuid | tr -d '-' > /etc/machine-id
fi

if [ ! -f "${PROTON_DIR}/proton" ]; then
 echo "[phase 1] Downloading ${PROTON_VERSION}..."
 wget -qO- "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/${PROTON_VERSION}/${PROTON_VERSION}.tar.gz" \
 | tar xzf - -C "${PROTON_DIR}" --strip-components=1
 ln -sfn "${PROTON_DIR}" "${STEAM_PATH}/compatibilitytools.d/${PROTON_VERSION}"
 echo "[phase 1] Proton installed."
else
 echo "[phase 1] Proton already cached."
fi

WINE64="${PROTON_DIR}/files/bin/wine64"

echo "[phase 2] Updating Deadlock server files..."
mkdir -p "${COMPAT_DATA}"
chown steam:steam "${INSTALL_DIR}" 2>/dev/null || true

if [ -f "${GAME_BASE_DIR}/game/bin/win64/deadlock.exe" ]; then
 echo "[phase 2] Game files already present in ${GAME_BASE_DIR} — skipping SteamCMD."
else
 if [ -z "${STEAM_LOGIN}" ] || [ -z "${STEAM_PASSWORD}" ]; then
 echo "[phase 2] ERROR: game files missing and STEAM_LOGIN/STEAM_PASSWORD unset"
 echo "[phase 2] Set GAMEFILES_MOUNT or provide Steam credentials in .env"
 exit 1
 fi
 mkdir -p "${GAME_BASE_DIR}"
 chown steam:steam "${GAME_BASE_DIR}"
 (
 flock -x 200
 MAX_RETRIES=3
 for attempt in $(seq 1 $MAX_RETRIES); do
 echo "[phase 2] SteamCMD attempt ${attempt}/${MAX_RETRIES}..."
 gosu steam /home/steam/steamcmd/steamcmd.sh \
 +@sSteamCmdForcePlatformType windows \
 +force_install_dir "$GAME_BASE_DIR" \
 +login "$STEAM_LOGIN" "$STEAM_PASSWORD" \
 +app_update "$APP_ID" \
 +quit && break
 echo "[phase 2] WARNING: attempt ${attempt} failed, retrying..."
 sleep 5
 done
 ) 200>"${GAME_BASE_DIR}/.download.lock"

 if [ ! -f "${GAME_BASE_DIR}/game/bin/win64/deadlock.exe" ]; then
 echo "[phase 2] ERROR: deadlock.exe not found after download"
 exit 1
 fi
fi
echo "[phase 2] Game files verified."

echo "[phase 2] Linking game files to instance directory..."
find "${INSTALL_DIR}" -xtype l -delete 2>/dev/null || true
gosu steam cp -rs "${GAME_BASE_DIR}/." "${INSTALL_DIR}/" 2>/dev/null || true

STEAM_CLIENT_DIR="/home/steam/steam_client"
if [ ! -f "${STEAM_CLIENT_DIR}/steamclient64.dll" ]; then
 echo "[phase 2] Downloading Windows Steam client DLLs..."
 mkdir -p "${STEAM_CLIENT_DIR}"
 gosu steam /home/steam/steamcmd/steamcmd.sh \
 +@sSteamCmdForcePlatformType windows \
 +force_install_dir "${STEAM_CLIENT_DIR}" \
 +login anonymous \
 +app_update 1007 validate \
 +quit || true
fi

Xvfb :99 -screen 0 640x480x24 &
XVFB_PID=$!
sleep 1

PROTON_MARKER="${PFXDIR}/.proton_marker"
if [ ! -f "${PROTON_MARKER}" ]; then
 echo "[phase 3] Initializing Wine prefix..."
 rm -rf "${PFXDIR}"
 gosu steam mkdir -p "${PFXDIR}"
 gosu steam bash -c "
 export WINEPREFIX='${PFXDIR}'
 export WINEDLLPATH='${PROTON_DIR}/files/lib64/wine/x86_64-windows:${PROTON_DIR}/files/lib64/wine/x86_64-unix'
 export DISPLAY=:99
 export WINEDEBUG=-all
 '${WINE64}' wineboot --init 2>&1
 " || true
 touch "${PROTON_MARKER}"
 echo "[phase 3] Wine prefix initialized."
else
 echo "[phase 3] Wine prefix already initialized."
fi

for dll in steamclient64.dll steamclient.dll; do
 SRC="${REDIST_DIR}/${dll}"
 [ ! -f "$SRC" ] && SRC="${STEAM_CLIENT_DIR}/${dll}"
 if [ -f "$SRC" ]; then
 mkdir -p "${PFXDIR}/drive_c/Program Files (x86)/Steam"
 cp -f "$SRC" "${PFXDIR}/drive_c/Program Files (x86)/Steam/${dll}"
 rm -f "${WIN64_DIR}/${dll}"
 cp -f "$SRC" "${WIN64_DIR}/${dll}"
 cp -f "$SRC" "${PFXDIR}/drive_c/windows/system32/${dll}"
 fi
done

DOTNET_WINE_DIR="${PFXDIR}/drive_c/Program Files/dotnet"
DOTNET_MARKER="${PFXDIR}/.dotnet_${DOTNET_VERSION}_marker"
DOTNET_CACHE="/opt/dotnet-cache"

if [ ! -f "${DOTNET_MARKER}" ]; then
 echo "[phase 4] Installing .NET ${DOTNET_VERSION} Windows runtime..."
 DOTNET_ZIP="${DOTNET_CACHE}/dotnet-runtime-${DOTNET_VERSION}-win-x64.zip"
 if [ ! -f "${DOTNET_ZIP}" ]; then
 mkdir -p "${DOTNET_CACHE}"
 wget -qO "${DOTNET_ZIP}" \
 "https://dotnetcli.azureedge.net/dotnet/Runtime/${DOTNET_VERSION}/dotnet-runtime-${DOTNET_VERSION}-win-x64.zip"
 fi
 mkdir -p "${DOTNET_WINE_DIR}"
 unzip -qo "${DOTNET_ZIP}" -d "${DOTNET_WINE_DIR}"
 touch "${DOTNET_MARKER}"
else
 echo "[phase 4] .NET ${DOTNET_VERSION} already installed."
fi

echo "[phase 5] Deploying deadworks..."
DEADWORKS_SRC="/opt/deadworks"
cp -f "${DEADWORKS_SRC}/game/bin/win64/deadworks.exe" "${WIN64_DIR}/"
rm -rf "${WIN64_DIR}/managed"
mkdir -p "${WIN64_DIR}/managed/plugins"
cp -rf "${DEADWORKS_SRC}/game/bin/win64/managed/"* "${WIN64_DIR}/managed/"

LIVE_PLUGINS_DIR="/opt/live-plugins"
if [ "${DEADWORKS_LIVE_PLUGINS:-0}" = "1" ]; then
 echo "[phase 5] Plugin hot-reload enabled -> watching ${LIVE_PLUGINS_DIR}"
 mkdir -p "${LIVE_PLUGINS_DIR}"
 chown steam:steam "${LIVE_PLUGINS_DIR}" 2>/dev/null || true
 rm -rf "${WIN64_DIR}/managed/plugins"
 ln -sfn "${LIVE_PLUGINS_DIR}" "${WIN64_DIR}/managed/plugins"
fi

mkdir -p "${INSTALL_DIR}/game/citadel/cfg"
rm -f "${INSTALL_DIR}/game/citadel/cfg/deadworks_mem.jsonc"
cp -f "${DEADWORKS_SRC}/game/citadel/cfg/deadworks_mem.jsonc" "${INSTALL_DIR}/game/citadel/cfg/"
rm -f "${WIN64_DIR}/steam_appid.txt" "${INSTALL_DIR}/game/citadel/steam_appid.txt"
echo "$APP_ID" > "${WIN64_DIR}/steam_appid.txt"
echo "$APP_ID" > "${INSTALL_DIR}/game/citadel/steam_appid.txt"
chown -Rh steam:steam "${WIN64_DIR}" "${INSTALL_DIR}/game/citadel"
chown -Rh steam:steam "${PFXDIR}"

if [ -d /match-instance/configs ]; then
 echo "[phase 5b] Applying PartyLock match config from /match-instance/configs ..."
 mkdir -p "${WIN64_DIR}/configs"
 cp -rf /match-instance/configs/. "${WIN64_DIR}/configs/"
 chown -Rh steam:steam "${WIN64_DIR}/configs"
fi

echo "[phase 5] Deadworks deployed."

SERVER_ARGS="-dedicated -console -dev -insecure -allow_no_lobby_connect -con_logfile console.log"
SERVER_ARGS="${SERVER_ARGS} +log 1 +ip 0.0.0.0 +sv_hibernate_when_empty 0"
SERVER_ARGS="${SERVER_ARGS} +hostport ${SERVER_PORT} +map ${SERVER_MAP} +tv_enable 0"
SERVER_ARGS="${SERVER_ARGS} +tv_citadel_auto_record 0 +spec_replay_enable 0 +citadel_upload_replay_enabled 0"

if [ -n "$MATCH_PUBLIC_HOST" ] && [ "$MATCH_PUBLIC_HOST" != "127.0.0.1" ] && [ "$MATCH_PUBLIC_HOST" != "localhost" ]; then
 SERVER_ARGS="${SERVER_ARGS} +net_public_adr ${MATCH_PUBLIC_HOST}"
fi

if [ -n "$SERVER_PASSWORD" ]; then
 SERVER_ARGS="${SERVER_ARGS} +sv_password ${SERVER_PASSWORD}"
fi
if [ -n "$RCON_PASSWORD" ]; then
 SERVER_ARGS="${SERVER_ARGS} +rcon_password ${RCON_PASSWORD}"
fi
if [ -n "$DEADWORKS_ARGS" ]; then
 SERVER_ARGS="${SERVER_ARGS} ${DEADWORKS_ARGS}"
fi

rm -f "${INSTALL_DIR}/game/citadel/console.log"

echo "[phase 6] Starting deadworks server on port ${SERVER_PORT}..."

PLUGIN_EXPORTS=""
while IFS='=' read -r key val; do
 PLUGIN_EXPORTS+="export ${key}='${val}'"$'\n'
done < <(env | grep -E '^(DEADWORKS_ENV_|PARTYLOCK_|MATCH_ORCHESTRATOR_INTERNAL_KEY)')

cat > /tmp/run_server.sh << SERVSCRIPT
#!/bin/bash
export STEAM_COMPAT_DATA_PATH='${COMPAT_DATA}'
export STEAM_COMPAT_CLIENT_INSTALL_PATH='${STEAM_PATH}'
export SteamAppId=${APP_ID}
export SteamGameId=${APP_ID}
export DISPLAY=:99
export WINEDEBUG=warn+module,err+all
export DOTNET_ROOT='C:\\Program Files\\dotnet'
${PLUGIN_EXPORTS}cd '${WIN64_DIR}'
'${PROTON_DIR}/proton' run ./deadworks.exe ${SERVER_ARGS} 2>&1
SERVSCRIPT
chmod +x /tmp/run_server.sh

CONSOLE_LOG="${INSTALL_DIR}/game/citadel/console.log"
mkdir -p "$(dirname "$CONSOLE_LOG")"
touch "$CONSOLE_LOG"
chown steam:steam "$CONSOLE_LOG"
tail -F "$CONSOLE_LOG" &
TAIL_PID=$!

gosu steam bash /tmp/run_server.sh &
SERVER_PID=$!
wait $SERVER_PID
EXIT_CODE=$?

kill "${TAIL_PID}" 2>/dev/null || true
kill "${XVFB_PID}" 2>/dev/null || true

echo "=== Server exited with code ${EXIT_CODE} ==="
if [ -f "$CONSOLE_LOG" ]; then
 tail -200 "$CONSOLE_LOG"
fi

exit $EXIT_CODE
