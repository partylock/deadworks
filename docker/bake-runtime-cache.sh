#!/bin/bash
# Bake Proton, Windows Steam client (1007), .NET zip, and Wine prefix seed into the image.
# Runtime entrypoint skips download/wineboot when these paths exist.
set -euo pipefail

PROTON_VERSION="${PROTON_VERSION:-GE-Proton10-33}"
DOTNET_VERSION="${DOTNET_VERSION:-10.0.0}"

PROTON_DIR="/opt/proton"
STEAM_CLIENT_DIR="/opt/steam_client_win"
DOTNET_CACHE="/opt/dotnet-cache"
WARM_PFX="/opt/compatdata-warm/pfx"
STEAM_PATH="/home/steam/.steam/steam"
WINE64="${PROTON_DIR}/files/bin/wine64"

steam_client_ready() {
 [ -f "${STEAM_CLIENT_DIR}/steamclient64.dll" ] \
 && [ "$(stat -c%s "${STEAM_CLIENT_DIR}/steamclient64.dll" 2>/dev/null || echo 0)" -gt 10000000 ]
}

install_steam_dll_to_prefix() {
 local dll="$1"
 local src="${STEAM_CLIENT_DIR}/${dll}"
 [ -f "$src" ] || return 0
 mkdir -p "${WARM_PFX}/drive_c/Program Files (x86)/Steam"
 cp -f "$src" "${WARM_PFX}/drive_c/Program Files (x86)/Steam/${dll}"
 cp -f "$src" "${WARM_PFX}/drive_c/windows/system32/${dll}"
}

echo "[bake] Proton ${PROTON_VERSION}..."
mkdir -p "${PROTON_DIR}" "${STEAM_PATH}/compatibilitytools.d"
if [ ! -f "${PROTON_DIR}/proton" ]; then
 wget -qO- "https://github.com/GloriousEggroll/proton-ge-custom/releases/download/${PROTON_VERSION}/${PROTON_VERSION}.tar.gz" \
 | tar xzf - -C "${PROTON_DIR}" --strip-components=1
fi
ln -sfn "${PROTON_DIR}" "${STEAM_PATH}/compatibilitytools.d/${PROTON_VERSION}"

echo "[bake] Windows Steam client (app 1007)..."
mkdir -p "${STEAM_CLIENT_DIR}"
chown -R steam:steam "${STEAM_CLIENT_DIR}"
if ! steam_client_ready; then
 gosu steam /home/steam/steamcmd/steamcmd.sh \
 +@sSteamCmdForcePlatformType windows \
 +force_install_dir "${STEAM_CLIENT_DIR}" \
 +login anonymous \
 +app_update 1007 validate \
 +quit
fi
if ! steam_client_ready; then
 echo "[bake] ERROR: steamclient64.dll missing after app 1007"
 exit 1
fi

echo "[bake] .NET ${DOTNET_VERSION} runtime zip..."
mkdir -p "${DOTNET_CACHE}"
DOTNET_ZIP="${DOTNET_CACHE}/dotnet-runtime-${DOTNET_VERSION}-win-x64.zip"
if [ ! -f "${DOTNET_ZIP}" ]; then
 wget -qO "${DOTNET_ZIP}" \
 "https://dotnetcli.azureedge.net/dotnet/Runtime/${DOTNET_VERSION}/dotnet-runtime-${DOTNET_VERSION}-win-x64.zip"
fi

echo "[bake] Wine prefix warm template (wineboot + .NET)..."
mkdir -p "${WARM_PFX}"
chown -R steam:steam /opt/compatdata-warm

Xvfb :99 -screen 0 640x480x24 &
XVFB_PID=$!
sleep 1

gosu steam bash -c "
export WINEPREFIX='${WARM_PFX}'
export WINEDLLPATH='${PROTON_DIR}/files/lib64/wine/x86_64-windows:${PROTON_DIR}/files/lib64/wine/x86_64-unix'
export DISPLAY=:99
export WINEDEBUG=-all
'${WINE64}' wineboot --init 2>&1
" || true

for dll in steamclient64.dll steamclient.dll tier0_s64.dll vstdlib_s64.dll; do
 install_steam_dll_to_prefix "${dll}"
done

DOTNET_WINE_DIR="${WARM_PFX}/drive_c/Program Files/dotnet"
mkdir -p "${DOTNET_WINE_DIR}"
unzip -qo "${DOTNET_ZIP}" -d "${DOTNET_WINE_DIR}"
if ! ls "${DOTNET_WINE_DIR}/host/fxr/"*/hostfxr.dll 1>/dev/null 2>&1; then
 echo "[bake] ERROR: hostfxr.dll missing after .NET unzip"
 exit 1
fi

touch "${WARM_PFX}/.proton_marker"
touch "${WARM_PFX}/.dotnet_${DOTNET_VERSION}_marker"
chown -R steam:steam /opt/compatdata-warm "${STEAM_CLIENT_DIR}" "${DOTNET_CACHE}" "${PROTON_DIR}" 2>/dev/null || true

kill "${XVFB_PID}" 2>/dev/null || true
echo "[bake] Runtime cache ready."
