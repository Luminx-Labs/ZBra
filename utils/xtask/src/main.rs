mod cli;
mod commands;
mod config;
mod context;

pub(crate) mod log;

use clap::Parser;

fn main() {
    let cli = cli::Cli::parse();
    if let Err(error) = commands::run(cli.command) {
        crate::error!("error: {error:#}");
        std::process::exit(1);
    }
}
