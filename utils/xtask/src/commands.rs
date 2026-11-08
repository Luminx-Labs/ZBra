mod abi;
mod build;
mod changelog;
mod clean;
mod dist;
mod doctor;
mod hooks;
mod release;
mod utils;
mod version;
mod workspace;

use crate::{cli::Command, context::Context};
use anyhow::Result;

pub(crate) fn run(command: Command) -> Result<()> {
    let context = Context::load()?;

    match command {
        Command::Ci => workspace::ci(&context),
        Command::Doctor { install } => doctor::run(&context, install),
        Command::Abi => abi::run(&context),
        Command::CheckWorkspace => workspace::check(&context),
        Command::Hooks => hooks::install(&context),
        Command::PreCommit => hooks::pre_commit(&context),
        Command::Clean => clean::run(&context),
        Command::Utils => utils::list(&context),
        Command::BuildUtils => utils::build(&context),
        Command::Changelog => changelog::run(&context),
        Command::Dist => dist::run(&context),
        Command::Version {
            set,
            bump,
            pre_release,
        } => version::run(&context, set.as_ref(), bump, pre_release),
        Command::Build => build::run(&context),
        Command::Release { release_version } => release::run(&context, release_version.as_ref()),
    }
}
