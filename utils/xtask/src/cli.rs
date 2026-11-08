use clap::{Parser, Subcommand, ValueEnum};
use semver::Version;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum VersionBump {
    Major,
    Minor,
    Patch,
}

#[derive(Debug, Parser)]
#[command(name = "cargo xtask", version, propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run all checks required by continuous integration.
    Ci,
    /// Check the development environment and configured tools.
    Doctor {
        /// Install missing package-manager dependencies when possible.
        #[arg(long)]
        install: bool,
    },
    /// Check Python and extension ABI compatibility.
    Abi,
    /// Check the Rust and Python workspaces.
    CheckWorkspace,
    /// Create CHANGELOG.md from Git history.
    Changelog,
    /// Install repository hooks.
    Hooks,
    /// Run the checks installed in the pre-commit hook.
    PreCommit,
    /// Remove Rust and Python build artifacts and caches.
    Clean,
    /// List configured native utility binaries.
    Utils,
    /// Build configured native utility binaries.
    BuildUtils,
    /// Build a portable distribution archive and checksum.
    Dist,
    /// Show or change the workspace version.
    Version {
        /// Set the workspace version, for example `0.2.0`.
        #[arg(long, conflicts_with = "bump")]
        set: Option<Version>,
        /// Increment the workspace version according to SemVer 2.0.
        #[arg(long, value_enum, conflicts_with = "set")]
        bump: Option<VersionBump>,
        /// Mark a bumped version as the first alpha prerelease.
        #[arg(long, conflicts_with = "set")]
        pre_release: bool,
    },
    /// Build the engine and supporting tools.
    Build,
    /// Create a release commit and annotated tag.
    Release {
        /// Version to release; defaults to the current workspace version.
        #[arg(long = "release")]
        release_version: Option<Version>,
    },
}
