use anyhow::{Context as _, Result, bail};
use serde::Deserialize;
use std::{fs, path::Path};

use crate::{error, info};
use std::process::Command;

fn run_cmd<I, S>(program: &str, args: I) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to spawn `{program}`: {e}"))
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) project: ProjectConfig,
    #[serde(default)]
    pub(crate) paths: PathsConfig,
    #[serde(default)]
    pub(crate) python: PythonConfig,
    #[serde(default)]
    pub(crate) model: ModelConfig,
    #[serde(default)]
    pub(crate) utils: Vec<UtilityConfig>,
    #[serde(default)]
    pub(crate) required_commands: Vec<Tool>,
}

#[derive(Debug, Deserialize)]
pub(crate) enum Tool {
    #[serde(rename = "cargo-llvm-cov")]
    Cov,
    #[serde(rename = "cargo-nextest")]
    Nextest,
    #[serde(rename = "taplo")]
    Taplo,
    #[serde(rename = "cargo-audit")]
    Audit,
    #[serde(rename = "cargo-deny")]
    Deny,
    #[serde(rename = "typos")]
    Typos,
}

impl Tool {
    pub(crate) fn install(&self) -> Result<()> {
        info!("checking if cargo binstall is installed");
        let binstall = Command::new("cargo")
            .arg("binstall")
            .arg("--version")
            .output();

        let is_binstall_installed = matches!(binstall, Ok(output) if output.status.success());

        if !is_binstall_installed {
            crate::info!("Installing cargo binstall as it not found");
            let status = Command::new("cargo")
                .args(["install", "cargo-binstall", "--locked"])
                .status()
                .with_context(|| "failed to install cargo-binstall")?;
            if !status.success() {
                bail!("failed to install cargo-binstall");
            }
        }
        let crate_name = match self {
            Tool::Cov => "cargo-llvm-cov",
            Tool::Nextest => "cargo-nextest",
            Tool::Taplo => "taplo-cli",
            Tool::Audit => "cargo-audit",
            Tool::Deny => "cargo-deny",
            Tool::Typos => "typos-cli",
        };

        info!("installing `{crate_name}` with cargo-binstall");
        let out = run_cmd("cargo", ["binstall", crate_name, "-y"])?;
        if !out.status.success() {
            bail!(
                "failed to install `{crate_name}`: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        Ok(())
    }
    pub(crate) fn check(&self) -> Result<()> {
        let (program, args, pretty) = match self {
            Tool::Nextest => ("cargo", &["nextest", "--version"][..], "cargo nextest"),
            Tool::Cov => ("cargo", &["llvm-cov", "--version"][..], "cargo llvm-cov"),
            Tool::Taplo => ("taplo", &["--version"][..], "taplo"),
            Tool::Audit => ("cargo", &["audit", "--version"][..], "cargo audit"),
            Tool::Deny => ("cargo", &["deny", "--version"][..], "cargo deny"),
            Tool::Typos => ("typos", &["--version"][..], "typos"),
        };

        let out = run_cmd(program, args)?;

        if out.status.success() {
            info!("found {pretty}");
        } else {
            error!(
                "command `{program} {args:?}` returned an error: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            bail!("`{pretty}` not functional");
        }

        Ok(())
    }
}

impl Config {
    pub(crate) fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        toml_edit::de::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ProjectConfig {
    pub(crate) name: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "zbra".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct PathsConfig {
    pub(crate) python_dir: String,
    pub(crate) model_dir: String,
    pub(crate) data_dir: String,
    pub(crate) engine_binary: String,
    pub(crate) hooks_dir: String,
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            python_dir: "python".to_owned(),
            model_dir: "models".to_owned(),
            data_dir: "data".to_owned(),
            engine_binary: "target/release/zbra".to_owned(),
            hooks_dir: ".githooks".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct PythonConfig {
    pub(crate) version: String,
    pub(crate) abi: String,
    pub(crate) implementation: String,
    pub(crate) package_manager: String,
    pub(crate) project_dir: String,
    pub(crate) virtual_environment: String,
    pub(crate) lockfile: String,
    #[serde(default)]
    pub(crate) required_tools: Vec<String>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            version: "3.12".to_owned(),
            abi: "cp312".to_owned(),
            implementation: "cpython".to_owned(),
            package_manager: "uv".to_owned(),
            project_dir: "python".to_owned(),
            virtual_environment: ".venv".to_owned(),
            lockfile: "uv.lock".to_owned(),
            required_tools: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelConfig {
    pub(crate) default: String,
    pub(crate) format_version: u32,
    pub(crate) require_checksum: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default: "models/current.nnue".to_owned(),
            format_version: 1,
            require_checksum: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct UtilityConfig {
    pub(crate) name: String,
    pub(crate) language: UtilityLanguage,
    pub(crate) path: String,
    pub(crate) binary: String,
    pub(crate) purpose: String,
    #[serde(default)]
    pub(crate) optional: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum UtilityLanguage {
    Rust,
    C,
}
