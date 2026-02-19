use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "claude-registry", about = "Claude Skills & Agents Registry CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage skills
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Manage agents
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },
    /// Build catalogs
    Catalog {
        #[command(subcommand)]
        action: CatalogAction,
    },
    /// Manage profiles
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// Install skills to a target project
    Install {
        /// Skill names to install
        #[arg(required = true)]
        names: Vec<String>,
        /// Target project path
        #[arg(long)]
        target: PathBuf,
        /// Preview without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// List installed skills
    List {
        /// Target project path
        #[arg(long)]
        target: PathBuf,
    },
    /// Create a new skill from template
    New {
        /// Skill name
        name: String,
        /// Skill description
        #[arg(long)]
        description: Option<String>,
    },
    /// Show available skills in registry
    Available,
    /// Uninstall a skill
    Uninstall {
        /// Skill name to uninstall
        name: String,
        /// Target project path
        #[arg(long)]
        target: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum AgentAction {
    /// Install agents to a target project
    Install {
        /// Agent names to install
        #[arg(required = true)]
        names: Vec<String>,
        /// Target project path
        #[arg(long)]
        target: PathBuf,
        /// Preview without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// List installed agents
    List {
        /// Target project path
        #[arg(long)]
        target: PathBuf,
    },
    /// Show available agents in registry
    Available,
    /// Uninstall an agent
    Uninstall {
        /// Agent name to uninstall
        name: String,
        /// Target project path
        #[arg(long)]
        target: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum CatalogAction {
    /// Build skill-catalog.json and agent-catalog.json
    Build,
}

#[derive(Subcommand)]
pub enum ProfileAction {
    /// Install all skills and agents from a profile
    Install {
        /// Profile name
        name: String,
        /// Target project path
        #[arg(long)]
        target: PathBuf,
        /// Preview without copying
        #[arg(long)]
        dry_run: bool,
    },
    /// List available profiles
    List,
}
