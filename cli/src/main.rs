mod catalog_builder;
mod cli;
mod commands;
mod frontmatter;
mod installer;
mod registry;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        cli::Commands::Skill { action } => commands::skill::run(action),
        cli::Commands::Agent { action } => commands::agent::run(action),
        cli::Commands::Catalog { action } => commands::catalog::run(action),
        cli::Commands::Profile { action } => commands::profile::run(action),
        cli::Commands::Hook { action } => commands::hook::run(action),
        cli::Commands::Complete { r#type } => commands::complete::run(r#type),
    }
}
