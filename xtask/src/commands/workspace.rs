use crate::context::Context;
use anyhow::Result;

pub(super) fn check(context: &Context) -> Result<()> {
    crate::info!("checking Rust formatting");
    context.cargo(&["fmt", "--all", "--", "--check"])?;
    crate::info!("checking the Rust workspace");
    context.cargo(&["check", "--workspace"])?;
    crate::info!("running Rust tests");
    context.cargo(&["test", "--workspace"])?;
    crate::info!("running Clippy with warnings denied");
    context.cargo(&[
        "clippy",
        "--workspace",
        "--all-targets",
        "--all-features",
        "--",
        "-D",
        "warnings",
    ])?;
    crate::success!("Rust workspace check passed");
    Ok(())
}

pub(super) fn ci(context: &Context) -> Result<()> {
    check(context)?;
    super::abi::run(context)
}
