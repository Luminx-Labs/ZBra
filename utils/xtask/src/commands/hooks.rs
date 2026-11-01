use crate::context::Context;
use anyhow::{Context as _, Result, bail};
use std::fs;

const PRE_COMMIT: &str = "#!/bin/sh
set -eu

cargo xtask pre-commit
";

pub(super) fn install(context: &Context) -> Result<()> {
    let hooks_dir = context.path(&context.config().paths.hooks_dir);
    fs::create_dir_all(&hooks_dir)
        .with_context(|| format!("failed to create {}", hooks_dir.display()))?;

    let hook = hooks_dir.join("pre-commit");
    if hook.exists() {
        let existing = fs::read_to_string(&hook)
            .with_context(|| format!("failed to read {}", hook.display()))?;
        if existing != PRE_COMMIT {
            bail!(
                "{} already exists and is not managed by xtask; refusing to overwrite it",
                hook.display()
            );
        }
    } else {
        fs::write(&hook, PRE_COMMIT)
            .with_context(|| format!("failed to write {}", hook.display()))?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(&hook)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&hook, permissions)?;
    }

    let hooks_path = context.config().paths.hooks_dir.as_str();
    context.run("git", &["config", "core.hooksPath", hooks_path])?;
    crate::success!("installed pre-commit hook at {}", hook.display());
    Ok(())
}

pub(super) fn pre_commit(context: &Context) -> Result<()> {
    super::workspace::check(context)?;
    crate::info!("checking Python and extension ABI");
    super::abi::run(context)?;
    crate::success!("pre-commit checks passed");
    Ok(())
}
