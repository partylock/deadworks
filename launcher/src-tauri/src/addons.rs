use std::collections::HashMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tokio::io::AsyncWriteExt;

const DEFAULT_API_URL: &str = match std::option_env!("DEADWORKS_API_URL") {
    Some(url) => url,
    None => "https://api.deadworks.net",
};

const VPK_MAGIC: [u8; 4] = [0x34, 0x12, 0xAA, 0x55]; // 0x55aa1234 LE

/// Hard cap on decompressed VPK size (4 GiB) to bound bz2-bomb damage.
const MAX_VPK_BYTES: u64 = 4 * 1024 * 1024 * 1024;

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub name: String,
    pub status: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub item_index: usize,
    pub total_items: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestItem {
    filename: String,
    kind: String, // "map" | "addon"
    version: u64,
    #[serde(default)]
    compressed_size: u64,
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct ContentManifest {
    items: Vec<ManifestItem>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
struct VersionEntry {
    kind: String,
    version: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct VersionsState {
    #[serde(default)]
    managed: HashMap<String, VersionEntry>, // filename → entry
}

// ── Validation ──

/// Reject filenames that could cause path traversal or absolute writes. Only
/// a single path component with no reserved characters is allowed.
fn validate_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("empty filename".into());
    }
    if name.len() > 128 {
        return Err(format!("filename too long: {}", name));
    }
    if name == "." || name == ".." {
        return Err(format!("invalid filename: {}", name));
    }
    for c in name.chars() {
        match c {
            '/' | '\\' | ':' | '\0' | '*' | '?' | '"' | '<' | '>' | '|' => {
                return Err(format!("filename contains reserved character: {}", name));
            }
            _ => {}
        }
    }
    Ok(())
}

// ── Helpers ──

fn find_game_dir() -> Result<PathBuf, String> {
    if let Some(override_dir) = crate::connect::get_game_dir_override() {
        return Ok(override_dir);
    }
    crate::connect::find_deadlock_game_dir()
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("Failed to create directory {}: {}", path.display(), e))
}

fn target_dir_for(kind: &str, game_dir: &Path, _filename: &str) -> Result<PathBuf, String> {
    let citadel = game_dir.join("citadel");
    match kind {
        "map" => Ok(citadel.join("maps")),
        "addon" => Ok(citadel.join("partylock_addons").join("vpks")),
        "skin" => Ok(citadel.join("partylock_skins").join("cache")),
        other => Err(format!("Unknown content kind: {}", other)),
    }
}

fn skin_cache_archive(game_dir: &Path, filename: &str, version: u64) -> PathBuf {
    game_dir
        .join("citadel")
        .join("partylock_skins")
        .join("cache")
        .join(format!("{}_v{}.7z", filename, version))
}

/// ponytail: Google Drive file id → direct download; large files may need confirm retry.
fn normalize_skin_download_url(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(id) = trimmed.strip_prefix("gdrive:") {
        return format!(
            "https://drive.google.com/uc?export=download&id={}",
            id.trim()
        );
    }
    if trimmed.contains("drive.google.com") {
        if let Some(id) = trimmed.split("id=").nth(1).and_then(|rest| rest.split('&').next()) {
            return format!(
                "https://drive.google.com/uc?export=download&id={}",
                id
            );
        }
    }
    trimmed.to_string()
}

fn is_7z_payload(bytes: &[u8]) -> bool {
    bytes.len() >= 6 && &bytes[..6] == b"7z\xBC\xAF\x27\x1C"
}

fn parse_gdrive_confirm_token(html: &str) -> Option<String> {
    for needle in ["confirm=", "confirm%3D"] {
        let rest = html.split(needle).nth(1)?;
        let token: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

fn extract_skin_7z(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    sevenz_rust::decompress_file(archive_path, dest_dir).map_err(|e| {
        format!(
            "Failed to extract skin archive {}: {}",
            archive_path.display(),
            e
        )
    })
}

async fn download_skin_archive(
    url: &str,
    dest_7z: &Path,
    item_name: &str,
    index: usize,
    total: usize,
    window: &tauri::Window,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let base_url = normalize_skin_download_url(url);
    let mut response = client
        .get(&base_url)
        .send()
        .await
        .map_err(|e| format!("Download request failed for {}: {}", item_name, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed for {}: HTTP {}",
            item_name,
            response.status()
        ));
    }

    let mut bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Download read failed for {}: {}", item_name, e))?
        .to_vec();

    if !is_7z_payload(&bytes) {
        let html = String::from_utf8_lossy(&bytes);
        if let Some(token) = parse_gdrive_confirm_token(&html) {
            let confirm_url = if base_url.contains('?') {
                format!("{base_url}&confirm={token}")
            } else {
                format!("{base_url}?confirm={token}")
            };
            response = client
                .get(&confirm_url)
                .send()
                .await
                .map_err(|e| format!("Google Drive confirm retry failed: {}", e))?;
            if !response.status().is_success() {
                return Err(format!(
                    "Google Drive confirm failed for {}: HTTP {}",
                    item_name,
                    response.status()
                ));
            }
            bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Download read failed for {}: {}", item_name, e))?
                .to_vec();
        }
    }

    if !is_7z_payload(&bytes) {
        return Err(format!(
            "Download for {} is not a .7z archive (check Google Drive link is public)",
            item_name
        ));
    }

    let total_bytes = bytes.len() as u64;
    let tmp = dest_7z.with_extension("7z.part");
    if let Some(parent) = tmp.parent() {
        ensure_dir(parent)?;
    }

    tokio::fs::write(&tmp, &bytes)
        .await
        .map_err(|e| format!("Failed to write temp 7z: {}", e))?;

    let _ = window.emit(
        "download-progress",
        DownloadProgress {
            name: item_name.to_string(),
            status: "downloading".into(),
            bytes_downloaded: total_bytes,
            total_bytes,
            item_index: index,
            total_items: total,
        },
    );

    tokio::fs::rename(&tmp, dest_7z)
        .await
        .map_err(|e| format!("Failed to finalize skin 7z: {}", e))
}

fn versions_path(game_dir: &Path) -> PathBuf {
    game_dir
        .join("citadel")
        .join("partylock_cache")
        .join("versions.json")
}

fn load_versions(game_dir: &Path) -> VersionsState {
    let path = versions_path(game_dir);
    std::fs::read(&path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<VersionsState>(&bytes).ok())
        .unwrap_or_default()
}

fn save_versions(game_dir: &Path, state: &VersionsState) -> Result<(), String> {
    let path = versions_path(game_dir);
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| format!("Failed to serialize versions.json: {}", e))?;
    std::fs::write(&path, &bytes)
        .map_err(|e| format!("Failed to write versions.json: {}", e))
}

fn verify_vpk_magic(path: &Path) -> Result<(), String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open {} for magic check: {}", path.display(), e))?;
    let mut buf = [0u8; 4];
    f.read_exact(&mut buf).map_err(|e| {
        format!("Failed to read VPK magic from {}: {}", path.display(), e)
    })?;
    if buf != VPK_MAGIC {
        return Err(format!(
            "decompressed payload for {} is not a valid VPK (magic mismatch)",
            path.display()
        ));
    }
    Ok(())
}

fn is_sharing_violation(err: &std::io::Error) -> bool {
    #[cfg(windows)]
    {
        // ERROR_SHARING_VIOLATION = 32, ERROR_ACCESS_DENIED = 5
        matches!(err.raw_os_error(), Some(32) | Some(5))
    }
    #[cfg(not(windows))]
    {
        matches!(err.kind(), std::io::ErrorKind::PermissionDenied)
    }
}

/// Resolve the manifest API URL from persisted settings rather than trusting
/// the webview to supply one. Production builds ignore the `local` endpoint.
fn resolve_api_url(app: &tauri::AppHandle) -> String {
    use tauri_plugin_store::StoreBuilder;
    if cfg!(debug_assertions) {
        if let Ok(store) = StoreBuilder::new(app, "settings.json").build() {
            let endpoint = store
                .get("api_endpoint")
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            if endpoint == "local" {
                return "http://localhost:8787".to_string();
            }
        }
    }
    DEFAULT_API_URL.to_string()
}

async fn fetch_manifest(api_url: &str, server_id: &str) -> Result<ContentManifest, String> {
    let url = format!("{}/api/servers/{}/content", api_url, server_id);
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("API request failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("API returned HTTP {}", resp.status()));
    }
    resp.json::<ContentManifest>()
        .await
        .map_err(|e| format!("Failed to parse manifest: {}", e))
}

/// Download `.vpk.bz2` from `url` into `dest_vpk` as a fully decompressed `.vpk`.
/// Enforces `MAX_VPK_BYTES` during decompression so a malicious manifest cannot
/// mount a bz2 bomb. Uses a temp `.part` file beside the destination, then
/// atomic rename.
async fn download_and_decompress(
    url: &str,
    dest_vpk: &Path,
    item_name: &str,
    index: usize,
    total: usize,
    expected_uncompressed_hint: u64,
    window: &tauri::Window,
) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Download request failed for {}: {}", item_name, e))?;
    if !response.status().is_success() {
        return Err(format!(
            "Download failed for {}: HTTP {}",
            item_name,
            response.status()
        ));
    }

    let total_compressed = response.content_length().unwrap_or(0);
    let bz2_tmp = dest_vpk.with_extension("vpk.bz2.part");

    // Stream compressed bytes to temp file
    {
        let mut file = tokio::fs::File::create(&bz2_tmp)
            .await
            .map_err(|e| format!("Failed to create temp file: {}", e))?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error for {}: {}", item_name, e))?;
            downloaded += chunk.len() as u64;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {}", e))?;
            let _ = window.emit(
                "download-progress",
                DownloadProgress {
                    name: item_name.to_string(),
                    status: "downloading".into(),
                    bytes_downloaded: downloaded,
                    total_bytes: total_compressed,
                    item_index: index,
                    total_items: total,
                },
            );
        }
        file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
    }

    // Decompress into a sibling .vpk.part file.
    let vpk_tmp = dest_vpk.with_extension("vpk.part");
    let bz2_tmp_clone = bz2_tmp.clone();
    let vpk_tmp_clone = vpk_tmp.clone();
    let name = item_name.to_string();
    let win = window.clone();

    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let input = std::fs::File::open(&bz2_tmp_clone)
            .map_err(|e| format!("Failed to open compressed temp: {}", e))?;
        let mut decoder = bzip2::read::BzDecoder::new(std::io::BufReader::new(input));
        let out_file = std::fs::File::create(&vpk_tmp_clone)
            .map_err(|e| format!("Failed to create {}: {}", vpk_tmp_clone.display(), e))?;
        let mut output = std::io::BufWriter::new(out_file);

        let mut buf = [0u8; 256 * 1024];
        let mut written: u64 = 0;
        loop {
            let n = decoder
                .read(&mut buf)
                .map_err(|e| format!("bz2 decompression failed for {}: {}", name, e))?;
            if n == 0 {
                break;
            }
            written += n as u64;
            if written > MAX_VPK_BYTES {
                return Err(format!(
                    "decompressed payload for {} exceeds maximum size ({} bytes)",
                    name, MAX_VPK_BYTES
                ));
            }
            std::io::Write::write_all(&mut output, &buf[..n])
                .map_err(|e| format!("Write error: {}", e))?;
            let _ = win.emit(
                "download-progress",
                DownloadProgress {
                    name: name.clone(),
                    status: "decompressing".into(),
                    bytes_downloaded: written,
                    total_bytes: expected_uncompressed_hint,
                    item_index: index,
                    total_items: total,
                },
            );
        }
        std::io::Write::flush(&mut output).map_err(|e| format!("Flush error: {}", e))?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Decompress task failed: {}", e))??;

    let _ = std::fs::remove_file(&bz2_tmp);

    verify_vpk_magic(&vpk_tmp)?;

    // Atomic rename onto the canonical path. This is the step that can fail
    // with a sharing violation if the engine has the old file open.
    match std::fs::rename(&vpk_tmp, dest_vpk) {
        Ok(()) => Ok(()),
        Err(e) if is_sharing_violation(&e) => {
            let _ = std::fs::remove_file(&vpk_tmp);
            Err(format!(
                "FILE_IN_USE: {} is currently loaded by Deadlock. Please fully disconnect or quit the game and try again.",
                dest_vpk
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default()
            ))
        }
        Err(e) => {
            let _ = std::fs::remove_file(&vpk_tmp);
            Err(format!("Failed to install {}: {}", dest_vpk.display(), e))
        }
    }
}

fn skin_mount_dir(game_dir: &Path) -> PathBuf {
    game_dir
        .join("citadel")
        .join("partylock_skins")
        .join("mount")
}

fn skin_addons_dir(game_dir: &Path) -> PathBuf {
    game_dir.join("citadel").join("addons")
}

fn active_skin_vpks_path(game_dir: &Path) -> PathBuf {
    game_dir
        .join("citadel")
        .join("partylock_skins")
        .join("active_vpks.json")
}

fn displaced_addon_vpks_path(game_dir: &Path) -> PathBuf {
    game_dir
        .join("citadel")
        .join("partylock_skins")
        .join("displaced_vpks.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisplacedAddonVpk {
    original: String,
    displaced: String,
}

/// ponytail: pak01–pak12 = PartyLock match slots; user addons get bumped to 13+ temporarily.
const SKIN_VPK_SLOT_START: u32 = 1;
const SKIN_VPK_SLOT_END: u32 = 12;
const DISPLACED_VPK_SLOT_START: u32 = 13;
const MAX_PAK_SLOT: u32 = 99;

fn pak_dir_vpk_name(slot: u32) -> String {
    format!("pak{slot:02}_dir.vpk")
}

fn parse_pak_dir_slot(filename: &str) -> Option<u32> {
    let lower = filename.to_lowercase();
    let num = lower.strip_prefix("pak")?.strip_suffix("_dir.vpk")?;
    if num.len() != 2 {
        return None;
    }
    num.parse().ok()
}

fn occupied_pak_slots(addons_dir: &Path) -> Result<std::collections::HashSet<u32>, String> {
    let mut occupied = std::collections::HashSet::new();
    if !addons_dir.is_dir() {
        return Ok(occupied);
    }

    for entry in std::fs::read_dir(addons_dir)
        .map_err(|e| format!("Failed to read {}: {}", addons_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        if !entry.file_type().map_err(|e| format!("Failed to read file type: {}", e))?.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(slot) = parse_pak_dir_slot(&name) {
            occupied.insert(slot);
        }
    }

    Ok(occupied)
}

fn next_free_pak_slot(
    occupied: &std::collections::HashSet<u32>,
    from: u32,
) -> Option<u32> {
    (from..=MAX_PAK_SLOT).find(|slot| !occupied.contains(slot))
}

fn load_displaced_addon_vpks(game_dir: &Path) -> Result<Vec<DisplacedAddonVpk>, String> {
    let path = displaced_addon_vpks_path(game_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse displaced VPK registry: {}", e))
}

fn save_displaced_addon_vpks(game_dir: &Path, entries: &[DisplacedAddonVpk]) -> Result<(), String> {
    let path = displaced_addon_vpks_path(game_dir);
    if entries.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
        }
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    let bytes = serde_json::to_vec_pretty(entries)
        .map_err(|e| format!("Failed to serialize displaced VPK registry: {}", e))?;
    std::fs::write(&path, bytes)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

/// Move user addon VPKs from pak01–12 into the first free slots from pak13 upward.
fn displace_competing_addon_vpks(game_dir: &Path) -> Result<(), String> {
    if !load_displaced_addon_vpks(game_dir)?.is_empty() {
        return Ok(());
    }

    let addons_dir = skin_addons_dir(game_dir);
    if !addons_dir.is_dir() {
        return Ok(());
    }

    let mut occupied = occupied_pak_slots(&addons_dir)?;
    let mut displaced = Vec::new();

    for slot in SKIN_VPK_SLOT_START..=SKIN_VPK_SLOT_END {
        let original = pak_dir_vpk_name(slot);
        let original_path = addons_dir.join(&original);
        if !original_path.is_file() {
            continue;
        }

        let target_slot = next_free_pak_slot(&occupied, DISPLACED_VPK_SLOT_START).ok_or_else(|| {
            format!(
                "No free addon slot between pak{:02} and pak{:02} to park existing mods",
                DISPLACED_VPK_SLOT_START, MAX_PAK_SLOT
            )
        })?;
        let displaced_name = pak_dir_vpk_name(target_slot);
        let displaced_path = addons_dir.join(&displaced_name);

        std::fs::rename(&original_path, &displaced_path).map_err(|e| {
            format!(
                "Failed to move {} -> {}: {}",
                original_path.display(),
                displaced_path.display(),
                e
            )
        })?;

        occupied.insert(target_slot);
        displaced.push(DisplacedAddonVpk {
            original,
            displaced: displaced_name,
        });
    }

    save_displaced_addon_vpks(game_dir, &displaced)
}

/// Restore addon VPKs parked in pak13+ back to their original pak01–12 names.
pub(crate) fn restore_displaced_addon_vpks(game_dir: &Path) -> Result<(), String> {
    let entries = load_displaced_addon_vpks(game_dir)?;
    if entries.is_empty() {
        return Ok(());
    }

    let addons_dir = skin_addons_dir(game_dir);
    let mut remaining = Vec::new();

    for entry in entries {
        let from = addons_dir.join(&entry.displaced);
        let to = addons_dir.join(&entry.original);

        if !from.is_file() {
            if to.is_file() {
                continue;
            }
            remaining.push(entry);
            continue;
        }

        if to.exists() {
            remaining.push(entry);
            continue;
        }

        std::fs::rename(&from, &to).map_err(|e| {
            format!(
                "Failed to restore {} -> {}: {}",
                from.display(),
                to.display(),
                e
            )
        })?;
    }

    save_displaced_addon_vpks(game_dir, &remaining)
}

fn prepare_partylock_vpk_slots(game_dir: &Path) -> Result<(), String> {
    restore_displaced_addon_vpks(game_dir)?;
    displace_competing_addon_vpks(game_dir)
}

fn clear_partylock_skin_vpks(game_dir: &Path) -> Result<(), String> {
    let registry = active_skin_vpks_path(game_dir);
    if registry.exists() {
        if let Ok(content) = std::fs::read_to_string(&registry) {
            if let Ok(files) = serde_json::from_str::<Vec<String>>(&content) {
                let addons = skin_addons_dir(game_dir);
                for name in files {
                    let path = addons.join(name);
                    if path.exists() {
                        std::fs::remove_file(&path).map_err(|e| {
                            format!("Failed to remove old skin VPK {}: {}", path.display(), e)
                        })?;
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&registry);
    }
    Ok(())
}

fn collect_vpk_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir).map_err(|e| format!("Failed to read {}: {}", dir.display(), e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to read file type: {}", e))?;

        if file_type.is_dir() {
            collect_vpk_files(&path, out)?;
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()).map(|ext| ext.eq_ignore_ascii_case("vpk")) == Some(true)
        {
            out.push(path);
        }
    }

    Ok(())
}

fn pick_largest_vpk(vpks: &[PathBuf]) -> Option<&PathBuf> {
    vpks.iter().max_by_key(|path| {
        std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
    })
}

fn copy_dir_merge(src: &Path, dest: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err(format!("Skin source is not a directory: {}", src.display()));
    }

    for entry in std::fs::read_dir(src).map_err(|e| format!("Failed to read {}: {}", src.display(), e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to read file type: {}", e))?;
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            ensure_dir(&dest_path)?;
            copy_dir_merge(&entry.path(), &dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                ensure_dir(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path).map_err(|e| {
                format!(
                    "Failed to copy {} -> {}: {}",
                    entry.path().display(),
                    dest_path.display(),
                    e
                )
            })?;
        }
    }

    Ok(())
}

fn extract_skin_archive_root(archive_path: &Path, staging_dir: &Path) -> Result<PathBuf, String> {
    if staging_dir.exists() {
        std::fs::remove_dir_all(staging_dir)
            .map_err(|e| format!("Failed to clear skin staging {}: {}", staging_dir.display(), e))?;
    }
    ensure_dir(staging_dir)?;
    extract_skin_7z(archive_path, staging_dir)?;

    let citadel_root = staging_dir.join("citadel");
    if citadel_root.is_dir() {
        return Ok(citadel_root);
    }

    Ok(staging_dir.to_path_buf())
}

fn rebuild_skin_mount(game_dir: &Path, items: &[ManifestItem]) -> Result<(), String> {
    clear_partylock_skin_vpks(game_dir)?;

    let mount = skin_mount_dir(game_dir);
    if mount.exists() {
        std::fs::remove_dir_all(&mount)
            .map_err(|e| format!("Failed to clear skin mount {}: {}", mount.display(), e))?;
    }
    ensure_dir(&mount)?;

    let addons_dir = skin_addons_dir(game_dir);
    ensure_dir(&addons_dir)?;

    let staging_parent = mount.join(".staging");
    if staging_parent.exists() {
        let _ = std::fs::remove_dir_all(&staging_parent);
    }

    let mut installed_vpks: Vec<String> = Vec::new();
    let mut used_loose_mount = false;
    let mut slots_prepared = false;

    for (idx, item) in items.iter().enumerate() {
        let cache_7z = skin_cache_archive(game_dir, &item.filename, item.version);
        if !cache_7z.exists() {
            return Err(format!(
                "Missing cached skin archive for {} (expected {})",
                item.filename,
                cache_7z.display()
            ));
        }

        let staging_dir = staging_parent.join(&item.filename);
        let content_root = extract_skin_archive_root(&cache_7z, &staging_dir)?;

        let mut vpks = Vec::new();
        collect_vpk_files(&content_root, &mut vpks)?;

        if let Some(src_vpk) = pick_largest_vpk(&vpks) {
            if !slots_prepared {
                prepare_partylock_vpk_slots(game_dir)?;
                slots_prepared = true;
            }

            let slot = SKIN_VPK_SLOT_START + idx as u32;
            if slot > SKIN_VPK_SLOT_END {
                return Err(format!(
                    "Too many skins in match (max {})",
                    SKIN_VPK_SLOT_END - SKIN_VPK_SLOT_START + 1
                ));
            }

            let dest_name = format!("pak{slot:02}_dir.vpk");
            let dest_vpk = addons_dir.join(&dest_name);
            verify_vpk_magic(src_vpk)?;
            std::fs::copy(src_vpk, &dest_vpk).map_err(|e| {
                format!(
                    "Failed to install skin VPK {} -> {}: {}",
                    src_vpk.display(),
                    dest_vpk.display(),
                    e
                )
            })?;
            installed_vpks.push(dest_name);
        } else {
            copy_dir_merge(&content_root, &mount)?;
            used_loose_mount = true;
        }
    }

    if !installed_vpks.is_empty() {
        let registry = active_skin_vpks_path(game_dir);
        if let Some(parent) = registry.parent() {
            ensure_dir(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(&installed_vpks)
            .map_err(|e| format!("Failed to serialize active skin VPK list: {}", e))?;
        std::fs::write(&registry, bytes)
            .map_err(|e| format!("Failed to write active skin VPK list: {}", e))?;
    }

    if staging_parent.exists() {
        let _ = std::fs::remove_dir_all(&staging_parent);
    }

    if !used_loose_mount && mount.read_dir().map(|mut d| d.next().is_none()).unwrap_or(true) {
        let _ = std::fs::remove_dir_all(&mount);
    }

    Ok(())
}

async fn download_skin_cache(
    url: &str,
    game_dir: &Path,
    filename: &str,
    version: u64,
    item_name: &str,
    index: usize,
    total: usize,
    window: &tauri::Window,
) -> Result<(), String> {
    let cache_7z = skin_cache_archive(game_dir, filename, version);

    let _ = window.emit(
        "download-progress",
        DownloadProgress {
            name: item_name.to_string(),
            status: "checking".into(),
            bytes_downloaded: 0,
            total_bytes: 0,
            item_index: index,
            total_items: total,
        },
    );

    if cache_7z.exists() {
        return Ok(());
    }

    download_skin_archive(url, &cache_7z, item_name, index, total, window).await
}

// ── Main command ──

/// Resolve PartyLock API base (`/api/v1`) from persisted settings.
fn resolve_partylock_api_url(app: &tauri::AppHandle) -> String {
    use tauri_plugin_store::StoreBuilder;
    let endpoint = StoreBuilder::new(app, "settings.json")
        .build()
        .ok()
        .and_then(|store| {
            store
                .get("api_endpoint")
                .and_then(|v| v.as_str().map(String::from))
        })
        .unwrap_or_else(|| "prod".to_string());

    if cfg!(debug_assertions) && endpoint == "local" {
        return "http://localhost:3001/api/v1".to_string();
    }

    std::env::var("PARTYLOCK_API_URL").unwrap_or_else(|_| {
        "http://localhost:3001/api/v1".to_string()
    })
}

fn read_access_token(app: &tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_store::StoreBuilder;
    let store = StoreBuilder::new(app, "settings.json")
        .build()
        .map_err(|e| format!("Failed to open settings store: {}", e))?;
    store
        .get("access_token")
        .and_then(|v| v.as_str().map(String::from))
        .ok_or_else(|| "Not logged in — open PartyLock launcher and sign in first.".into())
}

fn clear_installed_match_skins(game_dir: &Path) -> Result<(), String> {
    clear_partylock_skin_vpks(game_dir)?;
    restore_displaced_addon_vpks(game_dir)?;
    let mount = skin_mount_dir(game_dir);
    if mount.exists() {
        std::fs::remove_dir_all(&mount)
            .map_err(|e| format!("Failed to clear skin mount {}: {}", mount.display(), e))?;
    }
    Ok(())
}

fn read_skin_visibility_prefs(app: &tauri::AppHandle) -> (bool, bool) {
    use tauri_plugin_store::StoreBuilder;
    let store = StoreBuilder::new(app, "settings.json").build().ok();
    let hide_own = store
        .as_ref()
        .and_then(|s| s.get("hide_own_skin").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    let hide_others = store
        .as_ref()
        .and_then(|s| s.get("hide_others_skins").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    (hide_own, hide_others)
}

async fn fetch_match_manifest(
    api_url: &str,
    match_id: &str,
    token: &str,
    hide_own_skin: bool,
    hide_others_skins: bool,
) -> Result<ContentManifest, String> {
    let mut url = format!(
        "{}/draft/matches/{}/client-content",
        api_url.trim_end_matches('/'),
        match_id
    );
    let mut params = Vec::new();
    if hide_own_skin {
        params.push("hideOwnSkin=true");
    }
    if hide_others_skins {
        params.push("hideOthersSkins=true");
    }
    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("API returned HTTP {}", resp.status()));
    }
    resp.json::<ContentManifest>()
        .await
        .map_err(|e| format!("Failed to parse manifest: {}", e))
}

async fn install_manifest_items(
    window: &tauri::Window,
    manifest: ContentManifest,
    addr: &str,
    clear_match_skins_when_empty: bool,
) -> Result<crate::connect::ConnectResult, String> {
    for item in &manifest.items {
        validate_filename(&item.filename)?;
    }

    let has_addons = manifest.items.iter().any(|i| i.kind == "addon");
    let has_skins = manifest.items.iter().any(|i| i.kind == "skin");
    if has_addons || has_skins {
        let game_dir = find_game_dir()?;
        if has_addons {
            match crate::gameinfo::has_addonroot(&game_dir) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(
                        "gameinfo.gi is missing the addonroot entry required for client mods. \
                         Close Deadlock and restart the launcher to apply the patch."
                            .into(),
                    );
                }
                Err(e) => return Err(format!("Failed to check gameinfo.gi: {}", e)),
            }
        }
        if has_skins {
            match crate::gameinfo::has_skin_support(&game_dir) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(
                        "gameinfo.gi is missing skin search paths (citadel/addons and partylock mount). \
                         Feche o Deadlock completamente e reinicie o launcher para aplicar o patch."
                            .into(),
                    );
                }
                Err(e) => return Err(format!("Failed to check gameinfo.gi: {}", e)),
            }
        }
    }

    if manifest.items.is_empty() {
        return crate::connect::connect_to_server_inner(addr);
    }

    let game_dir = find_game_dir()?;
    let addons_dir = game_dir.join("citadel").join("partylock_addons").join("vpks");
    let maps_dir = game_dir.join("citadel").join("maps");
    let skins_cache_dir = game_dir.join("citadel").join("partylock_skins").join("cache");
    ensure_dir(&addons_dir)?;
    ensure_dir(&maps_dir)?;
    ensure_dir(&skins_cache_dir)?;

    let mut state = load_versions(&game_dir);
    let total_items = manifest.items.len();
    let mut skin_items: Vec<ManifestItem> = Vec::new();

    for (idx, item) in manifest.items.iter().enumerate() {
        let display_name = match item.kind.as_str() {
            "map" => format!("Map: {}", item.filename),
            "skin" => format!("Skin: {}", item.filename),
            _ => item.filename.clone(),
        };

        if item.kind == "skin" {
            skin_items.push(item.clone());
            let cache_7z = skin_cache_archive(&game_dir, &item.filename, item.version);
            let already_current = cache_7z.exists()
                && state
                    .managed
                    .get(&item.filename)
                    .map(|e| e.version == item.version && e.kind == item.kind)
                    .unwrap_or(false);

            if already_current {
                let _ = window.emit(
                    "download-progress",
                    DownloadProgress {
                        name: display_name.clone(),
                        status: "ready".into(),
                        bytes_downloaded: item.compressed_size,
                        total_bytes: item.compressed_size,
                        item_index: idx,
                        total_items,
                    },
                );
                continue;
            }

            download_skin_cache(
                &item.download_url,
                &game_dir,
                &item.filename,
                item.version,
                &display_name,
                idx,
                total_items,
                window,
            )
            .await?;

            state.managed.insert(
                item.filename.clone(),
                VersionEntry {
                    kind: item.kind.clone(),
                    version: item.version,
                },
            );
            save_versions(&game_dir, &state)?;

            let _ = window.emit(
                "download-progress",
                DownloadProgress {
                    name: display_name.clone(),
                    status: "ready".into(),
                    bytes_downloaded: item.compressed_size,
                    total_bytes: item.compressed_size,
                    item_index: idx,
                    total_items,
                },
            );
            continue;
        }

        let target_dir = target_dir_for(&item.kind, &game_dir, &item.filename)?;
        let vpk_filename = format!("{}.vpk", item.filename);
        let dest_vpk = target_dir.join(&vpk_filename);

        let already_current = dest_vpk.exists()
            && state
                .managed
                .get(&item.filename)
                .map(|e| e.version == item.version && e.kind == item.kind)
                .unwrap_or(false);

        if already_current {
            let _ = window.emit(
                "download-progress",
                DownloadProgress {
                    name: display_name.clone(),
                    status: "ready".into(),
                    bytes_downloaded: item.compressed_size,
                    total_bytes: item.compressed_size,
                    item_index: idx,
                    total_items,
                },
            );
            continue;
        }

        let _ = window.emit(
            "download-progress",
            DownloadProgress {
                name: display_name.clone(),
                status: "checking".into(),
                bytes_downloaded: 0,
                total_bytes: item.compressed_size,
                item_index: idx,
                total_items,
            },
        );

        download_and_decompress(
            &item.download_url,
            &dest_vpk,
            &display_name,
            idx,
            total_items,
            item.compressed_size.saturating_mul(3),
            window,
        )
        .await?;

        state.managed.insert(
            item.filename.clone(),
            VersionEntry {
                kind: item.kind.clone(),
                version: item.version,
            },
        );
        save_versions(&game_dir, &state)?;

        let _ = window.emit(
            "download-progress",
            DownloadProgress {
                name: display_name.clone(),
                status: "ready".into(),
                bytes_downloaded: item.compressed_size,
                total_bytes: item.compressed_size,
                item_index: idx,
                total_items,
            },
        );
    }

    if !skin_items.is_empty() {
        let _ = window.emit(
            "download-progress",
            DownloadProgress {
                name: "Montando skins".into(),
                status: "checking".into(),
                bytes_downloaded: 0,
                total_bytes: 0,
                item_index: 0,
                total_items,
            },
        );

        let game_dir_clone = game_dir.clone();
        let skins = skin_items.clone();
        tokio::task::spawn_blocking(move || rebuild_skin_mount(&game_dir_clone, &skins))
            .await
            .map_err(|e| format!("Skin mount task failed: {}", e))??;

        let _ = window.emit(
            "download-progress",
            DownloadProgress {
                name: "Skins prontas — reinicie o Deadlock se já estiver aberto".into(),
                status: "ready".into(),
                bytes_downloaded: 0,
                total_bytes: 0,
                item_index: 0,
                total_items,
            },
        );
    } else if clear_match_skins_when_empty {
        if let Ok(game_dir) = find_game_dir() {
            let game_dir_clone = game_dir.clone();
            tokio::task::spawn_blocking(move || clear_installed_match_skins(&game_dir_clone))
                .await
                .map_err(|e| format!("Skin clear task failed: {}", e))??;
        }
    }

    let _ = window.emit(
        "download-progress",
        serde_json::json!({ "name": "", "status": "connecting", "bytes_downloaded": 0, "total_bytes": 0, "item_index": 0, "total_items": 0 }),
    );

    crate::connect::connect_to_server_inner(addr)
}

#[tauri::command]
pub async fn prepare_and_connect(
    window: tauri::Window,
    server_id: String,
    addr: String,
) -> Result<crate::connect::ConnectResult, String> {
    let api_url = resolve_api_url(window.app_handle());

    let _ = window.emit(
        "download-progress",
        serde_json::json!({ "name": "", "status": "fetching", "bytes_downloaded": 0, "total_bytes": 0, "item_index": 0, "total_items": 0 }),
    );

    let manifest = fetch_manifest(&api_url, &server_id).await?;
    install_manifest_items(&window, manifest, &addr, false).await
}

#[tauri::command]
pub async fn prepare_and_connect_match(
    window: tauri::Window,
    match_id: String,
    addr: String,
) -> Result<crate::connect::ConnectResult, String> {
    let app = window.app_handle();
    let api_url = resolve_partylock_api_url(app);
    let token = read_access_token(app)?;

    let _ = window.emit(
        "download-progress",
        serde_json::json!({ "name": "", "status": "fetching", "bytes_downloaded": 0, "total_bytes": 0, "item_index": 0, "total_items": 0 }),
    );

    let (hide_own_skin, hide_others_skins) = read_skin_visibility_prefs(app);

    let manifest = fetch_match_manifest(
        &api_url,
        &match_id,
        &token,
        hide_own_skin,
        hide_others_skins,
    )
    .await?;
    install_manifest_items(&window, manifest, &addr, true).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_filename_with_separators() {
        assert!(validate_filename("a/b").is_err());
        assert!(validate_filename("a\\b").is_err());
        assert!(validate_filename("../etc/passwd").is_err());
        assert!(validate_filename("C:\\Windows\\foo").is_err());
    }

    #[test]
    fn rejects_filename_dots() {
        assert!(validate_filename(".").is_err());
        assert!(validate_filename("..").is_err());
    }

    #[test]
    fn rejects_empty_or_oversized_filename() {
        assert!(validate_filename("").is_err());
        assert!(validate_filename(&"a".repeat(129)).is_err());
    }

    #[test]
    fn accepts_plain_filename() {
        assert!(validate_filename("my_map_v2").is_ok());
        assert!(validate_filename("addon-1.2.3").is_ok());
    }

    #[test]
    fn parses_pak_dir_slots() {
        assert_eq!(parse_pak_dir_slot("pak01_dir.vpk"), Some(1));
        assert_eq!(parse_pak_dir_slot("PAK12_DIR.VPK"), Some(12));
        assert_eq!(parse_pak_dir_slot("pak1_dir.vpk"), None);
        assert_eq!(parse_pak_dir_slot("other.vpk"), None);
    }
}
