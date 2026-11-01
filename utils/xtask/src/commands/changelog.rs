use crate::context::Context;
use anyhow::{Context as _, Result};
use std::fs;

pub(super) fn run(context: &Context) -> Result<()> {
    let output = context.output("git", &["log", "--pretty=format:%h %s"])?;
    if !output.status.success() {
        anyhow::bail!("git log failed with status {}", output.status);
    }
    let commits = String::from_utf8(output.stdout).context("git returned non-UTF-8 output")?;
    let mut document = String::from("# Changelog\n\n");
    document.push_str("Generated from Git history by `cargo xtask changelog`.\n\n");
    document.push_str("## History\n\n");
    for commit in commits.lines() {
        document.push_str("- ");
        document.push_str(commit);
        document.push('\n');
    }

    let path = context.path("CHANGELOG.md");
    fs::write(&path, document).with_context(|| format!("failed to write {}", path.display()))?;
    crate::success!("wrote {}", path.display());
    Ok(())
}
