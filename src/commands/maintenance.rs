use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Commands for Spin maintenance tasks.
#[derive(Subcommand, Debug)]
pub enum MaintenanceCommands {
    /// Generate CLI reference docs in Markdown format.
    GenerateReference(GenerateReference),
}

impl MaintenanceCommands {
    pub async fn run(&self, app: clap::Command<'_>) -> anyhow::Result<()> {
        match self {
            MaintenanceCommands::GenerateReference(g) => g.run(app).await,
        }
    }
}

#[derive(Parser, Debug)]
pub struct GenerateReference {
    /// The file to which to generate the reference Markdown. If omitted, it is generated to stdout.
    #[clap(short = 'o')]
    pub output: Option<PathBuf>,
}

impl GenerateReference {
    pub async fn run(&self, app: clap::Command<'_>) -> anyhow::Result<()> {
        let markdown = clap_markdown::help_markdown_command(&app);
        match &self.output {
            Some(path) => std::fs::write(path, markdown)?,
            None => println!("{markdown}"),
        }
        Ok(())
    }
}
