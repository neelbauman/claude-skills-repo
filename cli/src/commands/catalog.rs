use anyhow::Result;

use crate::catalog_builder;
use crate::cli::CatalogAction;
use crate::registry;

pub fn run(action: CatalogAction) -> Result<()> {
    match action {
        CatalogAction::Build => build(),
    }
}

fn build() -> Result<()> {
    let root = registry::resolve_root()?;

    // Build skill catalog
    let skill_catalog = catalog_builder::build_skill_catalog(&root)?;
    let skill_path = root.join("skill-catalog.json");
    let skill_json = serde_json::to_string_pretty(&skill_catalog)?;
    std::fs::write(&skill_path, format!("{}\n", skill_json))?;
    println!(
        "Generated {} with {} skill(s)",
        skill_path.display(),
        skill_catalog.items.len()
    );

    // Build agent catalog
    let agent_catalog = catalog_builder::build_agent_catalog(&root)?;
    let agent_path = root.join("agent-catalog.json");
    let agent_json = serde_json::to_string_pretty(&agent_catalog)?;
    std::fs::write(&agent_path, format!("{}\n", agent_json))?;
    println!(
        "Generated {} with {} agent(s)",
        agent_path.display(),
        agent_catalog.items.len()
    );

    Ok(())
}
