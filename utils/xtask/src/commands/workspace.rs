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
    check_python(context)?;
    crate::success!("workspace checks passed");
    Ok(())
}

fn check_python(context: &Context) -> Result<()> {
    crate::info!("checking the Python lockfile");
    context.run("uv", &["lock", "--check"])?;

    crate::info!("checking Python lint");
    context.python_tool("ruff", &["check", "."])?;

    crate::info!("checking the Python test runner");
    context.python_tool("pytest", &["--version"])?;

    let tests = context.path("tests");
    if tests.is_dir() {
        crate::info!("running Python tests");
        context.python_tool("pytest", &[])?;
    } else {
        crate::warn!("no Python tests directory found; skipping pytest execution");
    }
    Ok(())
}

pub(super) fn ci(context: &Context) -> Result<()> {
    check(context)?;
    super::abi::run(context)
}
