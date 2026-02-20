use anyhow::{bail, Result};
use std::path::Path;

use crate::cli::HookAction;
use crate::installer;
use crate::registry;

pub fn run(action: HookAction) -> Result<()> {
    let root = registry::resolve_root()?;

    match action {
        HookAction::Install {
            name,
            global,
            target,
            dry_run,
        } => install(&root, &name, global, target.as_deref(), dry_run),
        HookAction::Uninstall {
            name,
            global,
            target,
        } => uninstall(&name, global, target.as_deref()),
        HookAction::List { global, target } => list(global, target.as_deref()),
        HookAction::Available => available(&root),
        HookAction::New { name, description } => new_hook(&root, &name, description),
    }
}

fn install(root: &Path, name: &str, global: bool, target: Option<&Path>, dry_run: bool) -> Result<()> {
    let hook_def = registry::hooks_dir(root)
        .join(name)
        .join("HOOK.json");

    if !hook_def.is_file() {
        bail!("Hook '{}' not found in registry", name);
    }

    let settings = registry::settings_path(global, target)?;
    installer::install_hook(&hook_def, &settings, dry_run)?;

    if dry_run {
        println!("\nDry run complete. No files were modified.");
    }
    Ok(())
}

fn uninstall(name: &str, global: bool, target: Option<&Path>) -> Result<()> {
    let settings = registry::settings_path(global, target)?;
    installer::uninstall_hook(name, &settings)
}

fn list(global: bool, target: Option<&Path>) -> Result<()> {
    let settings_path = registry::settings_path(global, target)?;

    if !settings_path.exists() {
        println!("No hooks installed (settings.json not found at {})", settings_path.display());
        return Ok(());
    }

    let content = std::fs::read_to_string(&settings_path)?;
    let v: serde_json::Value = serde_json::from_str(&content)?;

    let hooks_obj = match v.get("hooks").and_then(|h| h.as_object()) {
        Some(obj) => obj,
        None => {
            println!("No hooks installed in {}", settings_path.display());
            return Ok(());
        }
    };

    // Collect unique _registry_id values
    let mut ids: Vec<String> = Vec::new();
    for arr_val in hooks_obj.values() {
        if let Some(arr) = arr_val.as_array() {
            for entry in arr {
                if let Some(id) = entry.get("_registry_id").and_then(|v| v.as_str()) {
                    if !ids.contains(&id.to_string()) {
                        ids.push(id.to_string());
                    }
                }
            }
        }
    }

    if ids.is_empty() {
        println!("No registry-managed hooks in {}", settings_path.display());
    } else {
        println!("Installed hooks in {}:", settings_path.display());
        for id in &ids {
            println!("  {}", id);
        }
    }
    Ok(())
}

fn available(root: &Path) -> Result<()> {
    let hook_dirs = registry::list_hooks(root)?;

    println!("Available Hooks:");
    println!();
    for hook_dir in &hook_dirs {
        let name = hook_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let hook_json = hook_dir.join("HOOK.json");
        let desc = if hook_json.is_file() {
            let content = std::fs::read_to_string(&hook_json)?;
            let v: serde_json::Value = serde_json::from_str(&content)?;
            v.get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        };
        // Truncate long descriptions (char-boundary safe)
        let desc_short: String = if desc.chars().count() > 60 {
            let mut s: String = desc.chars().take(57).collect();
            s.push_str("...");
            s
        } else {
            desc
        };
        println!("  {:<24} {}", name, desc_short);
    }

    if hook_dirs.is_empty() {
        println!("  (none)");
    }
    Ok(())
}

fn new_hook(root: &Path, name: &str, description: Option<String>) -> Result<()> {
    let hook_dir = registry::hooks_dir(root).join(name);
    if hook_dir.exists() {
        bail!("Hook '{}' already exists", name);
    }

    let desc = description.unwrap_or_else(|| {
        "TODO: このフックの説明を書く。".to_string()
    });

    // Read template
    let template_path = registry::templates_dir(root).join("HOOK.json.template");
    let template = if template_path.is_file() {
        std::fs::read_to_string(&template_path)?
    } else {
        r#"{
  "name": "{{HOOK_NAME}}",
  "description": "{{DESCRIPTION}}",
  "tags": "",
  "hooks": {
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          { "type": "command", "command": "echo 'TODO'", "async": true }
        ]
      }
    ]
  }
}
"#
        .to_string()
    };

    let content = template
        .replace("{{HOOK_NAME}}", name)
        .replace("{{DESCRIPTION}}", &desc);

    std::fs::create_dir_all(hook_dir.join("scripts"))?;
    std::fs::write(hook_dir.join("HOOK.json"), content)?;
    std::fs::write(hook_dir.join("scripts").join(".gitkeep"), "")?;

    println!("Created claude/hooks/{}/", name);
    println!();
    println!("  claude/hooks/{}/", name);
    println!("  ├── HOOK.json");
    println!("  └── scripts/");
    println!();
    println!("Next steps:");
    println!("  1. claude/hooks/{}/HOOK.json を編集", name);
    println!("  2. claude-registry hook install {} --global でインストール", name);
    Ok(())
}
