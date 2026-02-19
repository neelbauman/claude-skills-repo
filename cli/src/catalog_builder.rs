use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

use crate::frontmatter::parse_frontmatter;
use crate::registry;

#[derive(Serialize)]
pub struct SkillCatalog {
    pub version: String,
    pub generated_at: String,
    pub items: Vec<SkillEntry>,
}

#[derive(Serialize)]
pub struct SkillEntry {
    pub name: String,
    pub dir: String,
    pub description: String,
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

#[derive(Serialize)]
pub struct AgentCatalog {
    pub version: String,
    pub generated_at: String,
    pub items: Vec<AgentEntry>,
}

#[derive(Serialize)]
pub struct AgentEntry {
    pub name: String,
    pub file: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tools: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub model: String,
}

pub fn build_skill_catalog(root: &Path) -> Result<SkillCatalog> {
    let now = chrono::Utc::now().to_rfc3339();
    let skill_dirs = registry::list_skills(root)?;
    let mut items = Vec::new();

    for skill_dir in skill_dirs {
        let skill_md = skill_dir.join("SKILL.md");
        let content = std::fs::read_to_string(&skill_md)?;
        let fm = parse_frontmatter(&content);

        let dir_name = skill_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let name = fm
            .get("name")
            .cloned()
            .unwrap_or_else(|| dir_name.clone());
        let description = fm.get("description").cloned().unwrap_or_default();
        let tags = fm
            .get("tags")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        // Collect files excluding .gitkeep
        let mut files = Vec::new();
        for entry in WalkDir::new(&skill_dir) {
            let entry = entry?;
            if entry.file_type().is_file() && entry.file_name() != ".gitkeep" {
                let rel = entry.path().strip_prefix(&skill_dir)?;
                files.push(rel.to_string_lossy().to_string());
            }
        }
        files.sort();

        items.push(SkillEntry {
            name,
            dir: dir_name,
            description,
            files,
            tags,
        });
    }

    Ok(SkillCatalog {
        version: "1.0.0".to_string(),
        generated_at: now,
        items,
    })
}

pub fn build_agent_catalog(root: &Path) -> Result<AgentCatalog> {
    let now = chrono::Utc::now().to_rfc3339();
    let agent_files = registry::list_agents(root)?;
    let mut items = Vec::new();

    for agent_file in agent_files {
        let content = std::fs::read_to_string(&agent_file)?;
        let fm = parse_frontmatter(&content);

        let file_name = agent_file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let stem = agent_file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let name = fm.get("name").cloned().unwrap_or_else(|| stem.clone());
        let description = fm.get("description").cloned().unwrap_or_default();
        let tags = fm
            .get("tags")
            .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();
        let tools = fm.get("tools").cloned().unwrap_or_default();
        let model = fm.get("model").cloned().unwrap_or_default();

        items.push(AgentEntry {
            name,
            file: file_name,
            description,
            tags,
            tools,
            model,
        });
    }

    Ok(AgentCatalog {
        version: "1.0.0".to_string(),
        generated_at: now,
        items,
    })
}
