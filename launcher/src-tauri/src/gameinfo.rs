use std::path::Path;

const ADDONROOT_VALUE: &str = "citadel/partylock_addons";
pub const ADDONS_GAME_VALUE: &str = "citadel/addons";
pub const SKIN_GAME_VALUE: &str = "citadel/partylock_skins/mount";

fn read_gameinfo(game_dir: &Path) -> Result<(std::path::PathBuf, String), String> {
    let gi_path = game_dir.join("citadel").join("gameinfo.gi");
    if !gi_path.exists() {
        return Err(format!("gameinfo.gi not found at {}", gi_path.display()));
    }
    let content = std::fs::read_to_string(&gi_path)
        .map_err(|e| format!("Failed to read gameinfo.gi: {}", e))?;
    Ok((gi_path, content))
}

fn line_indent(line: &str) -> &str {
    let stripped = line.trim_start();
    &line[..line.len().saturating_sub(stripped.len())]
}

fn is_commented(trimmed: &str) -> bool {
    trimmed.starts_with("//")
}

fn has_game_path_line(content: &str, value: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        !is_commented(trimmed) && trimmed.starts_with("Game") && trimmed.contains(value)
    })
}

/// First uncommented `Game citadel` line (not addons / mount / language paths).
fn find_base_game_citadel_line(lines: &[&str]) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_commented(trimmed) || !trimmed.starts_with("Game") {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        if parts.next()? != "Game" {
            continue;
        }
        if parts.next()? == "citadel" {
            return Some(i);
        }
    }
    None
}

fn insert_game_path_before_citadel(content: &str, value: &str) -> Result<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    if has_game_path_line(content, value) {
        return Ok(content.to_string());
    }

    let idx = find_base_game_citadel_line(&lines)
        .ok_or_else(|| "Could not find base Game citadel line in gameinfo.gi".to_string())?;
    let indent = line_indent(lines[idx]);
    let newline = if content.contains("\r\n") { "\r\n" } else { "\n" };

    let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    result.insert(idx, format!("{}Game\t{}", indent, value));
    Ok(result.join(newline))
}

/// Check if an uncommented line contains `addonroot` with our specific value.
fn has_addonroot_line(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        if is_commented(trimmed) {
            return false;
        }
        trimmed.starts_with("addonroot") && trimmed.contains(ADDONROOT_VALUE)
    })
}

/// Check if gameinfo.gi already contains the addonroot entry.
pub fn has_addonroot(game_dir: &Path) -> Result<bool, String> {
    let (_, content) = read_gameinfo(game_dir)?;
    Ok(has_addonroot_line(&content))
}

/// Check if gameinfo.gi mounts `citadel/addons` (required for .vpk skin mods).
pub fn has_addons_game_path(game_dir: &Path) -> Result<bool, String> {
    let (_, content) = read_gameinfo(game_dir)?;
    Ok(has_game_path_line(&content, ADDONS_GAME_VALUE))
}

/// Check if gameinfo.gi already mounts the PartyLock skin directory.
pub fn has_skin_search_path(game_dir: &Path) -> Result<bool, String> {
    let (_, content) = read_gameinfo(game_dir)?;
    Ok(has_game_path_line(&content, SKIN_GAME_VALUE))
}

/// Skins need both search paths: VPKs load from citadel/addons, loose files from mount.
pub fn has_skin_support(game_dir: &Path) -> Result<bool, String> {
    Ok(has_addons_game_path(game_dir)? && has_skin_search_path(game_dir)?)
}

/// Ensure gameinfo.gi has `addonroot citadel/partylock_addons` in its SearchPaths block.
pub fn ensure_addonroot(game_dir: &Path) -> Result<bool, String> {
    let (gi_path, content) = read_gameinfo(game_dir)?;

    let lines: Vec<&str> = content.lines().collect();
    let mut existing_idx: Option<usize> = None;
    let mut insert_idx: Option<usize> = None;
    let mut in_search_paths = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if is_commented(trimmed) {
            // fall through to SearchPaths tracking below
        } else if trimmed.starts_with("addonroot") {
            existing_idx = Some(i);
            break;
        }

        if trimmed.contains("SearchPaths") {
            in_search_paths = true;
            continue;
        }
        if in_search_paths && trimmed == "{" {
            insert_idx = Some(i + 1);
            in_search_paths = false;
        }
    }

    let newline = if content.contains("\r\n") { "\r\n" } else { "\n" };

    if let Some(i) = existing_idx {
        let line = lines[i];
        if line.contains(ADDONROOT_VALUE) {
            return Ok(false);
        }
        let indent = line_indent(line);
        let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
        result[i] = format!("{}addonroot\t{}", indent, ADDONROOT_VALUE);
        let new_content = result.join(newline);
        std::fs::write(&gi_path, &new_content)
            .map_err(|e| format!("Failed to write gameinfo.gi: {}", e))?;
        return Ok(true);
    }

    let idx = insert_idx
        .ok_or_else(|| "Could not find SearchPaths block in gameinfo.gi".to_string())?;

    let indent = lines
        .get(idx)
        .map(|line| line_indent(line))
        .unwrap_or("\t\t");

    let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();
    result.insert(idx, format!("{}addonroot\t{}", indent, ADDONROOT_VALUE));

    let new_content = result.join(newline);
    std::fs::write(&gi_path, &new_content)
        .map_err(|e| format!("Failed to write gameinfo.gi: {}", e))?;

    Ok(true)
}

/// Ensure `Game citadel/addons` is listed before the base `Game citadel` entry.
pub fn ensure_addons_game_path(game_dir: &Path) -> Result<bool, String> {
    let (gi_path, content) = read_gameinfo(game_dir)?;
    if has_game_path_line(&content, ADDONS_GAME_VALUE) {
        return Ok(false);
    }

    let new_content = insert_game_path_before_citadel(&content, ADDONS_GAME_VALUE)?;
    std::fs::write(&gi_path, &new_content)
        .map_err(|e| format!("Failed to write gameinfo.gi: {}", e))?;
    Ok(true)
}

/// Ensure `Game citadel/partylock_skins/mount` is listed before the base `Game citadel` entry.
pub fn ensure_skin_search_path(game_dir: &Path) -> Result<bool, String> {
    let (gi_path, content) = read_gameinfo(game_dir)?;
    if has_game_path_line(&content, SKIN_GAME_VALUE) {
        return Ok(false);
    }

    let new_content = insert_game_path_before_citadel(&content, SKIN_GAME_VALUE)?;
    std::fs::write(&gi_path, &new_content)
        .map_err(|e| format!("Failed to write gameinfo.gi: {}", e))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_base_game_citadel_line() {
        let sample = r#"
SearchPaths
{
    Game citadel/addons
    Game citadel
}
"#;
        let lines: Vec<&str> = sample.lines().collect();
        let idx = find_base_game_citadel_line(&lines).unwrap();
        assert!(lines[idx].contains("Game") && lines[idx].contains("citadel"));
        assert!(!lines[idx].contains("addons"));
    }

    #[test]
    fn inserts_addons_path_before_base_citadel() {
        let sample = "SearchPaths\n{\n\tGame\tcitadel\n}";
        let patched = insert_game_path_before_citadel(sample, ADDONS_GAME_VALUE).unwrap();
        let addons_pos = patched.find(ADDONS_GAME_VALUE).unwrap();
        let citadel_pos = patched.rfind("\tcitadel\n").unwrap();
        assert!(addons_pos < citadel_pos);
    }
}
