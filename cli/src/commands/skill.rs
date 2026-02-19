use anyhow::{bail, Result};
use std::path::Path;

use crate::cli::SkillAction;
use crate::frontmatter::parse_frontmatter;
use crate::installer;
use crate::registry;

pub fn run(action: SkillAction) -> Result<()> {
    let root = registry::resolve_root()?;

    match action {
        SkillAction::Install {
            names,
            target,
            dry_run,
        } => install(&root, &names, &target, dry_run),
        SkillAction::List { target } => list(&target),
        SkillAction::New { name, description } => new_skill(&root, &name, description),
        SkillAction::Available => available(&root),
        SkillAction::Uninstall { name, target } => uninstall(&name, &target),
    }
}

fn install(root: &Path, names: &[String], target: &Path, dry_run: bool) -> Result<()> {
    let skills_dir = registry::skills_dir(root);
    let target_skills = target.join(".claude").join("skills");

    println!(
        "Installing {} skill(s) to {}",
        names.len(),
        target_skills.display()
    );

    for name in names {
        let src = skills_dir.join(name);
        let dst = target_skills.join(name);
        if let Err(e) = installer::install_skill(&src, &dst, dry_run) {
            eprintln!("  Warning: {}", e);
        }
    }

    if dry_run {
        println!("\nDry run complete. No files were copied.");
    } else {
        println!("\nDone! Skills installed to {}", target_skills.display());
    }
    Ok(())
}

fn list(target: &Path) -> Result<()> {
    let target_skills = target.join(".claude").join("skills");
    if !target_skills.is_dir() {
        println!("No skills installed at {}", target_skills.display());
        return Ok(());
    }

    println!("Installed skills in {}:", target_skills.display());
    for entry in std::fs::read_dir(&target_skills)? {
        let entry = entry?;
        if entry.path().is_dir() {
            println!("  {}", entry.file_name().to_string_lossy());
        }
    }
    Ok(())
}

fn new_skill(root: &Path, name: &str, description: Option<String>) -> Result<()> {
    let skill_dir = registry::skills_dir(root).join(name);
    if skill_dir.exists() {
        bail!("Skill '{}' already exists", name);
    }

    let desc = description.unwrap_or_else(|| {
        "TODO: このスキルの説明を書く。Claudeがいつこのスキルを使うべきか判断できるように具体的に。".to_string()
    });

    // Read template
    let template_path = registry::templates_dir(root).join("SKILL.md.template");
    let template = if template_path.is_file() {
        std::fs::read_to_string(&template_path)?
    } else {
        "---\nname: {{SKILL_NAME}}\ndescription: >\n  {{DESCRIPTION}}\ntags: \n---\n\n# {{SKILL_NAME}}\n".to_string()
    };

    let content = template
        .replace("{{SKILL_NAME}}", name)
        .replace("{{DESCRIPTION}}", &desc);

    // Create directory structure
    std::fs::create_dir_all(skill_dir.join("scripts"))?;
    std::fs::create_dir_all(skill_dir.join("references"))?;
    std::fs::create_dir_all(skill_dir.join("assets"))?;

    std::fs::write(skill_dir.join("SKILL.md"), content)?;

    // .gitkeep for empty dirs
    for sub in &["scripts", "references", "assets"] {
        std::fs::write(skill_dir.join(sub).join(".gitkeep"), "")?;
    }

    println!("Created claude/skills/{}/", name);
    println!();
    println!("  claude/skills/{}/", name);
    println!("  ├── SKILL.md");
    println!("  ├── scripts/");
    println!("  ├── references/");
    println!("  └── assets/");
    println!();
    println!("Next steps:");
    println!("  1. claude/skills/{}/SKILL.md を編集", name);
    println!("  2. claude-registry catalog build でカタログ更新");
    Ok(())
}

fn available(root: &Path) -> Result<()> {
    let skill_dirs = registry::list_skills(root)?;

    println!("Available Skills:");
    println!();
    for skill_dir in &skill_dirs {
        let name = skill_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let skill_md = skill_dir.join("SKILL.md");
        let desc = if skill_md.is_file() {
            let content = std::fs::read_to_string(&skill_md)?;
            let fm = parse_frontmatter(&content);
            fm.get("description").cloned().unwrap_or_default()
        } else {
            String::new()
        };
        println!("  {:<24} {}", name, desc);
    }

    if skill_dirs.is_empty() {
        println!("  (none)");
    }
    Ok(())
}

fn uninstall(name: &str, target: &Path) -> Result<()> {
    let target_path = target.join(".claude").join("skills").join(name);
    installer::uninstall_skill(&target_path)
}
