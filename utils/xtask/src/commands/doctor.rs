use crate::context::Context;
use anyhow::{Result, bail};

/// Result of checking everything `doctor` cares about.
struct ToolStatus {
    /// Indices into `config().required_commands` that failed `check()`.
    missing_commands: Vec<usize>,
    package_manager_available: bool,
    /// Python tools that failed `--version` (all of them if the package manager is missing).
    missing_python_tools: Vec<String>,
}

impl ToolStatus {
    fn is_ok(&self) -> bool {
        self.missing_commands.is_empty()
            && self.package_manager_available
            && self.missing_python_tools.is_empty()
    }
}

pub(super) fn run(context: &Context, install: bool) -> Result<()> {
    crate::info!("project: {}", context.config().project.name);
    crate::info!("workspace: {}", context.root().display());
    context.cargo(&["--version"])?;
    context.python(&["--version"])?;
    context.run("git", &["--version"])?;

    // Phase 1: check.
    crate::info!("checking the required tools according to .xtask.toml");
    let mut status = check_tools(context);

    // Phase 2: install only what is missing, then check again.
    if !status.is_ok() && install {
        report_missing(context, &status);
        crate::info!("installing missing tools");
        install_missing(context, &status)?;

        crate::info!("re-checking tools after installation");
        status = check_tools(context);
    }

    // Phase 3: fail if anything is still missing.
    if !status.is_ok() {
        report_missing(context, &status);
        if install {
            bail!("some tools are still missing after installation");
        }
        bail!("one or more required tools are missing; run `cargo xtask doctor --install`");
    }
    crate::success!("all required tools are available");

    crate::info!("checking Python ABI pin");
    super::abi::run(context)?;

    print_summary(context);
    Ok(())
}

/// Checks everything without installing or failing early.
fn check_tools(context: &Context) -> ToolStatus {
    let config = context.config();

    let missing_commands = config
        .required_commands
        .iter()
        .enumerate()
        .filter(|(_, tool)| tool.check().is_err())
        .map(|(index, _)| index)
        .collect();

    let package_manager = &config.python.package_manager;
    let package_manager_available = context.command_available(package_manager);
    if package_manager_available {
        crate::success!("Python package manager available: {package_manager}");
    } else {
        crate::warn!("Python package manager `{package_manager}` is unavailable");
    }

    // Python tools live in the project environment, so they can only be
    // checked once the package manager exists.
    let missing_python_tools = if package_manager_available {
        config
            .python
            .required_tools
            .iter()
            .filter(|tool| match context.python_tool(tool, &["--version"]) {
                Ok(_) => {
                    crate::success!("Python tool available: {tool}");
                    false
                }
                Err(error) => {
                    crate::warn!("Python tool `{tool}` is unavailable: {error:#}");
                    true
                }
            })
            .cloned()
            .collect()
    } else {
        config.python.required_tools.clone()
    };

    ToolStatus {
        missing_commands,
        package_manager_available,
        missing_python_tools,
    }
}

fn report_missing(context: &Context, status: &ToolStatus) {
    if !status.missing_commands.is_empty() {
        crate::warn!(
            "{} required command(s) missing",
            status.missing_commands.len()
        );
    }
    if !status.package_manager_available {
        crate::warn!(
            "Python package manager `{}` missing",
            context.config().python.package_manager
        );
    }
    if !status.missing_python_tools.is_empty() {
        crate::warn!(
            "Python tools missing: {}",
            status.missing_python_tools.join(", ")
        );
    }
}

/// Installs only what `status` reports as missing.
fn install_missing(context: &Context, status: &ToolStatus) -> Result<()> {
    if !status.package_manager_available {
        install_package_manager(context)?;
    }

    if !status.package_manager_available || !status.missing_python_tools.is_empty() {
        sync_python_environment(context)?;
    }

    for &index in &status.missing_commands {
        context.config().required_commands[index].install()?;
    }

    Ok(())
}

fn install_package_manager(context: &Context) -> Result<()> {
    let package_manager = &context.config().python.package_manager;

    match package_manager.as_str() {
        "uv" => {
            crate::info!("installing uv");
            let installed = std::process::Command::new("cargo")
                .args(["binstall", "uv", "-y"])
                .status()
                .is_ok_and(|status| status.success());
            if !installed {
                crate::info!("cargo binstall failed; falling back to pip");
                context.python(&["-m", "pip", "install", "--user", "uv"])?;
            }
        }
        "pip" => {
            crate::info!("Python pip is managed by the Python installation");
        }
        other => {
            bail!("cannot install unsupported Python package manager `{other}` automatically")
        }
    }

    if !context.command_available(package_manager) {
        bail!("failed to make Python package manager `{package_manager}` available");
    }
    Ok(())
}

fn sync_python_environment(context: &Context) -> Result<()> {
    let package_manager = &context.config().python.package_manager;
    let python_project = context.path(&context.config().python.project_dir);

    if !python_project.join("pyproject.toml").is_file() {
        return Ok(());
    }

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
    Ok(())
}

fn print_summary(context: &Context) {
    let config = context.config();

    crate::info!("configured Python: {}", config.python.version);
    crate::info!("Python executable: {}", context.python_command());
    crate::info!("Python package manager: {}", config.python.package_manager);
    crate::info!("configured Python ABI: {}", config.python.abi);
    crate::info!("Python implementation: {}", config.python.implementation);
    crate::info!(
        "Python project directory: {}",
        context.path(&config.python.project_dir).display()
    );
    crate::info!(
        "Python virtual environment: {}",
        context.path(&config.python.virtual_environment).display()
    );
    crate::info!(
        "Python lockfile: {}",
        context.path(&config.python.lockfile).display()
    );
    crate::info!(
        "default model: {} (format {})",
        context.path(&config.model.default).display(),
        config.model.format_version
    );
    crate::info!(
        "engine binary: {}",
        context.path(&config.paths.engine_binary).display()
    );
    crate::info!(
        "Python directory: {}",
        context.path(&config.paths.python_dir).display()
    );
    crate::info!(
        "model directory: {}",
        context.path(&config.paths.model_dir).display()
    );
    crate::info!(
        "data directory: {}",
        context.path(&config.paths.data_dir).display()
    );
    crate::info!(
        "model checksum required: {}",
        config.model.require_checksum
    );
}
