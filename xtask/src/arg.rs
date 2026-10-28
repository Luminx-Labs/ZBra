use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", version, propagate_version = true)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run CI Checks
    Ci,
    /// Check the Environment & Tools
    Doctor,
    /// Check the Workspace
    CheckWorkspace,
    /// Create a Changelog Markdown File based on Git history
    Changelog,
    /// Install Hooks
    Hooks,
    /// Build archives and checksums for the distribution binary
    Dist,
    /// Show the version derived from the latest release tag, or synchronize manifests
    Version,
    /// Build the Engine and/or other utility(s)
    Build,
    /// Create a release commit and annotated tag
    Release,
}

pub fn run() -> Result<()> {
    match Args::parse().command {
        Command::Ci => todo!(),
        Command::Doctor => todo!(),
        Command::CheckWorkspace => todo!(),
        Command::Changelog => todo!(),
        Command::Hooks => todo!(),
        Command::Dist => todo!(),
        Command::Version => todo!(),
        Command::Build => todo!(),
        Command::Release => todo!(),
    }
}
