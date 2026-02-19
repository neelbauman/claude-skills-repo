use anyhow::Result;
use std::path::Path;

use crate::cli::AgentAction;
use crate::frontmatter::parse_frontmatter;
use crate::installer;
use crate::registry;

pub fn run(action: AgentAction) -> Result<()> {
    let root = registry::resolve_root()?;

    match action {
        AgentAction::Install {
            names,
            target,
            dry_run,
        } => install(&root, &names, &target, dry_run),
        AgentAction::List { target } => list(&target),
        AgentAction::Available => available(&root),
        AgentAction::Uninstall { name, target } => uninstall(&root, &name, &target),
    }
}

fn install(root: &Path, names: &[String], target: &Path, dry_run: bool) -> Result<()> {
    let agents_dir = registry::agents_dir(root);
    let target_agents = target.join(".claude").join("agents");

    println!(
        "Installing {} agent(s) to {}",
        names.len(),
        target_agents.display()
    );

    if !dry_run {
        std::fs::create_dir_all(&target_agents)?;
    }

    for name in names {
        let src = agents_dir.join(format!("{}.md", name));
        let dst = target_agents.join(format!("{}.md", name));
        if let Err(e) = installer::install_agent(&src, &dst, dry_run) {
            eprintln!("  Warning: {}", e);
        }
    }

    if dry_run {
        println!("\nDry run complete. No files were copied.");
    } else {
        println!("\nDone! Agents installed to {}", target_agents.display());
    }
    Ok(())
}

fn list(target: &Path) -> Result<()> {
    let target_agents = target.join(".claude").join("agents");
    if !target_agents.is_dir() {
        println!("No agents installed at {}", target_agents.display());
        return Ok(());
    }

    println!("Installed agents in {}:", target_agents.display());
    for entry in std::fs::read_dir(&target_agents)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |e| e == "md") {
            let name = path.file_stem().unwrap_or_default().to_string_lossy();
            println!("  {}", name);
        }
    }
    Ok(())
}

fn available(root: &Path) -> Result<()> {
    let agent_files = registry::list_agents(root)?;

    println!("Available Agents:");
    println!();
    for agent_file in &agent_files {
        let name = agent_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let content = std::fs::read_to_string(agent_file)?;
        let fm = parse_frontmatter(&content);
        let desc = fm.get("description").cloned().unwrap_or_default();
        // Truncate long descriptions for display (char-boundary safe)
        let desc_short: String = if desc.chars().count() > 80 {
            let mut s: String = desc.chars().take(77).collect();
            s.push_str("...");
            s
        } else {
            desc
        };
        println!("  {:<24} {}", name, desc_short);
    }

    if agent_files.is_empty() {
        println!("  (none)");
    }
    Ok(())
}

fn uninstall(_root: &Path, name: &str, target: &Path) -> Result<()> {
    let target_path = target
        .join(".claude")
        .join("agents")
        .join(format!("{}.md", name));
    installer::uninstall_agent(&target_path)
}
