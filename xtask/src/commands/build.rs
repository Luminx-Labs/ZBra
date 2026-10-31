use crate::context::Context;
use anyhow::Result;

pub(super) fn run(context: &Context) -> Result<()> {
    crate::info!("building the Rust workspace in release mode");
    context.cargo(&["build", "--workspace", "--release"])?;
    crate::info!("building configured native utilities");
    super::utils::build(context)?;
    crate::success!("release build completed");
    Ok(())
}
