use crate::context::Context;
use anyhow::{Result, bail};

pub(super) fn run(context: &Context, install: bool) -> Result<()> {
    if install {
        install_dependencies(context)?;
    }

    crate::info!("project: {}", context.config().project.name);
    crate::info!("workspace: {}", context.root().display());
    context.cargo(&["--version"])?;
    context.python(&["--version"])?;
    context.run("git", &["--version"])?;

    let package_manager = &context.config().python.package_manager;
    if context.command_available(package_manager) {
        crate::success!("Python package manager available: {package_manager}");
    } else {
        bail!(
            "Python package manager `{package_manager}` is unavailable; run `cargo xtask doctor --install`"
        );
    }

    crate::info!("configured Rust: {}", context.config().environment.rust);
    crate::info!("configured Python: {}", context.config().python.version);
    crate::info!("Python executable: {}", context.python_command());
    crate::info!(
        "Python package manager: {}",
        context.config().python.package_manager
    );
    crate::info!("configured Python ABI: {}", context.config().python.abi);
    crate::info!(
        "Python implementation: {}",
        context.config().python.implementation
    );
    crate::info!(
        "Python project directory: {}",
        context.path(&context.config().python.project_dir).display()
    );
    crate::info!(
        "Python virtual environment: {}",
        context
            .path(&context.config().python.virtual_environment)
            .display()
    );
    crate::info!(
        "Python lockfile: {}",
        context.path(&context.config().python.lockfile).display()
    );

    match &context.config().environment.cuda {
        Some(cuda) => {
            crate::info!("configured CUDA: {cuda}");
        }
        None => {
            crate::warn!("configured CUDA: not specified");
        }
    }

    match context.config().python.pytorch.as_deref() {
        None => {
            crate::warn!("configured PyTorch: not specified");
        }
        Some(pytorch) => {
            crate::info!("configured PyTorch: {pytorch}");
        }
    }

    crate::info!("tracked libraries: {}", context.config().libraries.len());
    for library in &context.config().libraries {
        crate::info!(
            "library: {} {} ({}, purpose: {}, optional: {}, ABI: {})",
            library.name,
            library.version,
            library.ecosystem,
            library.purpose,
            library.optional,
            library.python_abi.as_deref().unwrap_or("n/a")
        );
    }

    crate::info!("tracked models: {}", context.config().models.len());
    for model in &context.config().models {
        crate::info!(
            "model: {} {} [{}; {}; format {}; sha256: {}; dataset: {}; commit: {}] -> {}",
            model.name,
            model.version,
            model.status,
            model.architecture,
            model.format_version,
            model.sha256.as_deref().unwrap_or("not recorded"),
            model.dataset.as_deref().unwrap_or("not recorded"),
            model.training_commit.as_deref().unwrap_or("not recorded"),
            context.path(&model.path).display()
        );
    }

    crate::info!(
        "default model: {} (format {})",
        context.path(&context.config().model.default).display(),
        context.config().model.format_version
    );
    crate::info!(
        "engine binary: {}",
        context
            .path(&context.config().paths.engine_binary)
            .display()
    );
    crate::info!(
        "Python directory: {}",
        context.path(&context.config().paths.python_dir).display()
    );
    crate::info!(
        "model directory: {}",
        context.path(&context.config().paths.model_dir).display()
    );
    crate::info!(
        "data directory: {}",
        context.path(&context.config().paths.data_dir).display()
    );
    crate::info!(
        "model checksum required: {}",
        context.config().model.require_checksum
    );
    Ok(())
}

fn install_dependencies(context: &Context) -> Result<()> {
    let package_manager = &context.config().python.package_manager;
    if !context.command_available(package_manager) {
        match package_manager.as_str() {
            "uv" => {
                crate::info!("installing uv with Python pip");
                context.python(&["-m", "pip", "install", "--user", "uv"])?;
            }
            "pip" => {
                crate::info!("Python pip is managed by the Python installation");
            }
            other => {
                bail!("cannot install unsupported Python package manager `{other}` automatically")
            }
        }
    }

    if !context.command_available(package_manager) {
        bail!("failed to make Python package manager `{package_manager}` available");
    }

    let python_project = context.path(&context.config().python.project_dir);
    if python_project.join("pyproject.toml").is_file() {
        crate::info!("synchronizing Python environment with {package_manager}");
        match package_manager.as_str() {
            "uv" => context.run(
                "uv",
                &["sync", "--project", &python_project.display().to_string()],
            )?,
            "pip" => crate::warn!(
                "pip project synchronization is not configured; install from its lockfile manually"
            ),
            _ => crate::warn!("project synchronization is not configured for {package_manager}"),
        }
    }

    Ok(())
}
