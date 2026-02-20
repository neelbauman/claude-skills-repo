use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolve the registry root directory.
/// Priority: CLAUDE_REGISTRY_ROOT env var > current directory.
pub fn resolve_root() -> Result<PathBuf> {
    if let Ok(root) = std::env::var("CLAUDE_REGISTRY_ROOT") {
        let p = PathBuf::from(root);
        if p.is_dir() {
            return Ok(p);
        }
    }
    std::env::current_dir().context("Failed to get current directory")
}

/// Return path to skills directory: <root>/claude/skills
pub fn skills_dir(root: &Path) -> PathBuf {
    root.join("claude").join("skills")
}

/// Return path to agents directory: <root>/claude/agents
pub fn agents_dir(root: &Path) -> PathBuf {
    root.join("claude").join("agents")
}

/// Return path to profiles directory: <root>/profiles
pub fn profiles_dir(root: &Path) -> PathBuf {
    root.join("profiles")
}

/// Return path to templates directory: <root>/templates
pub fn templates_dir(root: &Path) -> PathBuf {
    root.join("templates")
}

/// List skill directories (each containing SKILL.md)
pub fn list_skills(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = skills_dir(root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut results = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            results.push(path);
        }
    }
    results.sort();
    Ok(results)
}

/// List agent markdown files in agents directory
pub fn list_agents(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = agents_dir(root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut results = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            results.push(path);
        }
    }
    results.sort();
    Ok(results)
}

/// Return path to hooks directory: <root>/claude/hooks
pub fn hooks_dir(root: &Path) -> PathBuf {
    root.join("claude").join("hooks")
}

/// List hook directories (each containing HOOK.json)
pub fn list_hooks(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = hooks_dir(root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut results = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() && path.join("HOOK.json").is_file() {
            results.push(path);
        }
    }
    results.sort();
    Ok(results)
}

/// Resolve settings.json path.
/// global=true → ~/.claude/settings.json
/// target=Some(p) → p/.claude/settings.json
pub fn settings_path(global: bool, target: Option<&Path>) -> anyhow::Result<std::path::PathBuf> {
    if global {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(PathBuf::from(home).join(".claude").join("settings.json"))
    } else if let Some(t) = target {
        Ok(t.join(".claude").join("settings.json"))
    } else {
        anyhow::bail!("Either --global or --target must be specified")
    }
}

/// List profile JSON files
pub fn list_profiles(root: &Path) -> Result<Vec<PathBuf>> {
    let dir = profiles_dir(root);
    if !dir.is_dir() {
        return Ok(vec![]);
    }
    let mut results = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "json") {
            results.push(path);
        }
    }
    results.sort();
    Ok(results)
}
