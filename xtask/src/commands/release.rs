use crate::context::Context;
use anyhow::{Result, bail};
use semver::Version;

pub(super) fn run(context: &Context, requested: Option<&Version>) -> Result<()> {
    let version = requested
        .cloned()
        .unwrap_or(super::version::current(context)?);
    let tag = format!("v{version}");

    check_clean_worktree(context)?;
    let tag_exists = context.output(
        "git",
        &["rev-parse", "--verify", &format!("refs/tags/{tag}")],
    )?;
    if tag_exists.status.success() {
        bail!("Git tag {tag} already exists");
    }

    crate::info!("running release checks");
    super::hooks::pre_commit(context)?;

    crate::info!("updating release files for {version}");
    super::version::run(context, Some(&version), None)?;
    super::changelog::run(context)?;
    super::build::run(context)?;
    super::hooks::pre_commit(context)?;

    context.run("git", &["add", "Cargo.toml", "Cargo.lock", "CHANGELOG.md"])?;
    context.run("git", &["commit", "-m", &format!("chore(release): {tag}")])?;
    context.run("git", &["tag", "-a", &tag, "-m", &format!("Release {tag}")])?;
    crate::success!("created release {tag}");
    Ok(())
}

fn check_clean_worktree(context: &Context) -> Result<()> {
    let status = context.output("git", &["status", "--porcelain"])?;
    if !status.status.success() {
        bail!("git status failed with status {}", status.status);
    }
    if !status.stdout.is_empty() {
        bail!("release requires a clean Git worktree");
    }
    Ok(())
}
