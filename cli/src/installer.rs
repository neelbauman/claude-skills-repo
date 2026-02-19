use anyhow::{bail, Context, Result};
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
