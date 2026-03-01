//! Configuration persistence commands
//!
//! Save/load/list/delete configuration profiles as JSON files
//! in the platform-appropriate app data directory.

use crate::domain::Configuration;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Get (and create if needed) the configs directory.
/// Think of this like Python's `os.makedirs(path, exist_ok=True)` — it ensures
/// the directory exists and returns the path.
fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    let dir = base.join("configs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create configs dir: {e}"))?;
    Ok(dir)
}

/// Sanitize a configuration name to prevent path traversal.
/// Like Python's `os.path.basename()` check — rejects anything with
/// path separators, "..", or empty strings.
fn sanitize_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Configuration name cannot be empty".to_string());
    }
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err("Invalid configuration name".to_string());
    }
    // Only allow alphanumeric, spaces, hyphens, underscores
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_')
    {
        return Err("Configuration name contains invalid characters".to_string());
    }
    Ok(trimmed.to_string())
}

#[tauri::command]
pub fn save_configuration(app: AppHandle, config: Configuration) -> Result<(), String> {
    let name = sanitize_name(&config.name)?;
    let dir = config_dir(&app)?;
    let path = dir.join(format!("{name}.json"));
    let json =
        serde_json::to_string_pretty(&config).map_err(|e| format!("Serialization error: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write config: {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn load_configuration(app: AppHandle, name: String) -> Result<Configuration, String> {
    let name = sanitize_name(&name)?;
    let dir = config_dir(&app)?;
    let path = dir.join(format!("{name}.json"));
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config '{name}': {e}"))?;
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse config '{name}': {e}"))
}

#[tauri::command]
pub fn list_configurations(app: AppHandle) -> Result<Vec<String>, String> {
    let dir = config_dir(&app)?;
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| format!("Failed to read configs dir: {e}"))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()?.to_str()? == "json" {
                path.file_stem()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    Ok(names)
}

#[tauri::command]
pub fn delete_configuration(app: AppHandle, name: String) -> Result<(), String> {
    let name = sanitize_name(&name)?;
    if name == "Default" {
        return Err("Cannot delete the Default configuration".to_string());
    }
    let dir = config_dir(&app)?;
    let path = dir.join(format!("{name}.json"));
    if !path.exists() {
        return Err(format!("Configuration '{name}' not found"));
    }
    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete config '{name}': {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_name_rejects_path_traversal() {
        assert!(sanitize_name("../evil").is_err());
        assert!(sanitize_name("foo/bar").is_err());
        assert!(sanitize_name("foo\\bar").is_err());
        assert!(sanitize_name("").is_err());
        assert!(sanitize_name("  ").is_err());
    }

    #[test]
    fn sanitize_name_accepts_valid_names() {
        assert_eq!(sanitize_name("Default").unwrap(), "Default");
        assert_eq!(sanitize_name("FT-991A Home").unwrap(), "FT-991A Home");
        assert_eq!(sanitize_name("my_config_2").unwrap(), "my_config_2");
    }

    #[test]
    fn sanitize_name_rejects_special_characters() {
        assert!(sanitize_name("config<>").is_err());
        assert!(sanitize_name("config;drop").is_err());
        assert!(sanitize_name("config|pipe").is_err());
    }
}
