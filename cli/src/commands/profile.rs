use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::Path;

use crate::cli::ProfileAction;
use crate::installer;
use crate::registry;

#[derive(Deserialize)]
struct Profile {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    agents: Vec<String>,
}

pub fn run(action: ProfileAction) -> Result<()> {
    let root = registry::resolve_root()?;

    match action {
        ProfileAction::Install {
            name,
            target,
            dry_run,
        } => install(&root, &name, &target, dry_run),
        ProfileAction::List => list(&root),
    }
}

fn install(root: &Path, name: &str, target: &Path, dry_run: bool) -> Result<()> {
    let profile_path = registry::profiles_dir(root).join(format!("{}.json", name));
    if !profile_path.is_file() {
        bail!("Profile '{}' not found", name);
    }

    let content = std::fs::read_to_string(&profile_path)?;
    let profile: Profile = serde_json::from_str(&content)?;

    let total = profile.skills.len() + profile.agents.len();
    println!(
        "Installing profile '{}' ({} item(s)) to {}",
        profile.name,
        total,
        target.display()
    );

    // Install skills
    if !profile.skills.is_empty() {
        let skills_dir = registry::skills_dir(root);
        let target_skills = target.join(".claude").join("skills");

        for skill_name in &profile.skills {
            let src = skills_dir.join(skill_name);
            let dst = target_skills.join(skill_name);
            if let Err(e) = installer::install_skill(&src, &dst, dry_run) {
                eprintln!("  Warning: {}", e);
            }
        }
    }

    // Install agents
    if !profile.agents.is_empty() {
        let agents_dir = registry::agents_dir(root);
        let target_agents = target.join(".claude").join("agents");

        if !dry_run {
            std::fs::create_dir_all(&target_agents)?;
        }

        for agent_name in &profile.agents {
            let src = agents_dir.join(format!("{}.md", agent_name));
            let dst = target_agents.join(format!("{}.md", agent_name));
            if let Err(e) = installer::install_agent(&src, &dst, dry_run) {
                eprintln!("  Warning: {}", e);
            }
        }
    }

    if dry_run {
        println!("\nDry run complete. No files were copied.");
    } else {
        println!("\nDone!");
    }
    Ok(())
}

fn list(root: &Path) -> Result<()> {
    let profiles = registry::list_profiles(root)?;

    println!("Available Profiles:");
    println!();
    for profile_path in &profiles {
        let content = std::fs::read_to_string(profile_path)?;
        let profile: Profile = serde_json::from_str(&content)?;
        let name = profile_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        println!("  {:<24} {}", name, profile.description);

        if !profile.skills.is_empty() {
            println!("    skills: {}", profile.skills.join(", "));
        }
        if !profile.agents.is_empty() {
            println!("    agents: {}", profile.agents.join(", "));
        }
    }

    if profiles.is_empty() {
        println!("  (none)");
    }
    Ok(())
}
