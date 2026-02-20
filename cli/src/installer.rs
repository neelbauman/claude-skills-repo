use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Copy a skill directory to the target, excluding .gitkeep files.
pub fn install_skill(src: &Path, dst: &Path, dry_run: bool) -> Result<()> {
    let name = src
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if !src.is_dir() {
        bail!("Skill '{}' not found in registry", name);
    }

    if dry_run {
        println!("  [dry-run] Would install skill: {}", name);
        return Ok(());
    }

    fs::create_dir_all(dst).with_context(|| format!("Failed to create {}", dst.display()))?;

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;

        // Skip .gitkeep
        if entry.file_name() == ".gitkeep" {
            continue;
        }

        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }

    println!("  Installed skill: {}", name);
    Ok(())
}

/// Copy an agent markdown file to the target.
pub fn install_agent(src: &Path, dst: &Path, dry_run: bool) -> Result<()> {
    let name = src
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if !src.is_file() {
        bail!("Agent '{}' not found in registry", name);
    }

    if dry_run {
        println!("  [dry-run] Would install agent: {}", name);
        return Ok(());
    }

    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    println!("  Installed agent: {}", name);
    Ok(())
}

/// Remove a skill directory from the target.
pub fn uninstall_skill(target: &Path) -> Result<()> {
    let name = target
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if !target.is_dir() {
        bail!("Skill '{}' is not installed", name);
    }

    fs::remove_dir_all(target)?;
    println!("  Uninstalled skill: {}", name);
    Ok(())
}

// ─── Hook installer ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct HookDefinition {
    name: String,
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
    tags: String,
    hooks: Map<String, Value>,
}

/// Load settings.json; return empty map if the file does not exist.
fn load_settings(path: &Path) -> Result<Map<String, Value>> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let v: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    match v {
        Value::Object(m) => Ok(m),
        _ => bail!("{} is not a JSON object", path.display()),
    }
}

/// Write settings map back to disk (pretty-printed, UTF-8).
fn save_settings(path: &Path, settings: &Map<String, Value>) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let content = serde_json::to_string_pretty(&Value::Object(settings.clone()))?;
    fs::write(path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

/// Install a hook definition into settings.json (idempotent).
///
/// For each event key in HOOK.json, existing entries tagged with
/// `_registry_id == name` are removed, then the new entries (with the tag
/// appended) are added.
pub fn install_hook(hook_def_path: &Path, settings_path: &Path, dry_run: bool) -> Result<()> {
    let content = fs::read_to_string(hook_def_path)
        .with_context(|| format!("Failed to read {}", hook_def_path.display()))?;
    let def: HookDefinition = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", hook_def_path.display()))?;

    if dry_run {
        println!("  [dry-run] Would install hook '{}' into {}", def.name, settings_path.display());
        for (event, groups) in &def.hooks {
            println!("    event: {} ({} group(s))", event, groups.as_array().map_or(0, |a| a.len()));
        }
        return Ok(());
    }

    let mut settings = load_settings(settings_path)?;

    // Ensure top-level "hooks" key exists as an object
    let hooks_obj = settings
        .entry("hooks")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .context("settings.json 'hooks' field is not an object")?;

    for (event, new_groups_val) in &def.hooks {
        let new_groups = new_groups_val
            .as_array()
            .with_context(|| format!("hooks.{} must be an array", event))?;

        // Get or create the array for this event
        let arr = hooks_obj
            .entry(event.clone())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .with_context(|| format!("hooks.{} is not an array in settings.json", event))?;

        // Remove existing entries for this registry id
        arr.retain(|entry| {
            entry
                .get("_registry_id")
                .and_then(|v| v.as_str())
                .map_or(true, |id| id != def.name)
        });

        // Append new entries with _registry_id tag
        for group in new_groups {
            let mut g = group.clone();
            if let Some(obj) = g.as_object_mut() {
                obj.insert("_registry_id".to_string(), Value::String(def.name.clone()));
            }
            arr.push(g);
        }
    }

    save_settings(settings_path, &settings)?;
    println!(
        "  Installed hook '{}' into {}",
        def.name,
        settings_path.display()
    );
    println!("  description: {}", def.description);
    Ok(())
}

/// Remove all hook entries tagged with `name` from settings.json.
pub fn uninstall_hook(name: &str, settings_path: &Path) -> Result<()> {
    if !settings_path.exists() {
        bail!("Hook '{}' is not installed (settings.json not found)", name);
    }

    let mut settings = load_settings(settings_path)?;

    let hooks_obj = match settings.get_mut("hooks").and_then(|v| v.as_object_mut()) {
        Some(obj) => obj,
        None => bail!("Hook '{}' is not installed (no hooks in settings.json)", name),
    };

    let mut removed = 0usize;
    for arr_val in hooks_obj.values_mut() {
        if let Some(arr) = arr_val.as_array_mut() {
            let before = arr.len();
            arr.retain(|entry| {
                entry
                    .get("_registry_id")
                    .and_then(|v| v.as_str())
                    .map_or(true, |id| id != name)
            });
            removed += before - arr.len();
        }
    }

    if removed == 0 {
        bail!("Hook '{}' is not installed", name);
    }

    save_settings(settings_path, &settings)?;
    println!("  Uninstalled hook '{}' from {}", name, settings_path.display());
    Ok(())
}

/// Remove an agent file from the target.
pub fn uninstall_agent(target: &Path) -> Result<()> {
    let name = target
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if !target.is_file() {
        bail!("Agent '{}' is not installed", name);
    }

    fs::remove_file(target)?;
    println!("  Uninstalled agent: {}", name);
    Ok(())
}
