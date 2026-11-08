use crate::context::Context;
use anyhow::Result;

pub(super) fn check(context: &Context) -> Result<()> {
    crate::info!("checking Rust formatting");
    context.cargo(&["fmt", "--all", "--", "--check"])?;
    crate::info!("checking TOML formatting");
    context.run("taplo", &["fmt", "--check"])?;
    crate::info!("auditing dependencies");
    context.cargo(&["audit"])?;
    crate::info!("checking the Rust workspace");
    context.cargo(&["check", "--workspace"])?;
    crate::info!("running Rust tests with nextest");
    context.cargo(&["nextest", "run", "--workspace"])?;
    crate::info!("checking Rust coverage instrumentation");
    context.cargo(&["llvm-cov", "--workspace", "--all-features", "--no-report"])?;
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
    crate::info!("checking the dependencies with deny");
    context.cargo(&["deny", "check", "all"])?;
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
