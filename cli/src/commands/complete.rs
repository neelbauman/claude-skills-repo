// implements: SPEC001
use anyhow::Result;

use crate::cli::CompletionType;
use crate::registry;

pub fn run(completion_type: CompletionType) -> Result<()> {
    let root = registry::resolve_root()?;

    match completion_type {
        CompletionType::Skills => {
            for path in registry::list_skills(&root)? {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    println!("{}", name);
                }
            }
        }
        CompletionType::Agents => {
            for path in registry::list_agents(&root)? {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    println!("{}", name);
                }
            }
        }
        CompletionType::Profiles => {
            for path in registry::list_profiles(&root)? {
                if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                    println!("{}", name);
                }
            }
        }
        CompletionType::Hooks => {
            for path in registry::list_hooks(&root)? {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    println!("{}", name);
                }
            }
        }
    }

    Ok(())
}
