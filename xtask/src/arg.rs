use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", version, propagate_version = true)]
pub struct Arg {
    commands: Commands,
}

#[derive(Clone, Debug)]
pub enum Commands {
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
        Commands::Ci => todo!(),
        Commands::Doctor => todo!(),
        Commands::CheckWorkspace => todo!(),
        Commands::Changelog => todo!(),
        Commands::Hooks => todo!(),
        Commands::Dist => todo!(),
        Commands::Version => todo!(),
        Commands::Build => todo!(),
        Commands::Release => todo!(),
    }
}
