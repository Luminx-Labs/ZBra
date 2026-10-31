use crate::config::Config;
use anyhow::{Context as _, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use semver::Version;
use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

pub struct Context {
    root: PathBuf,
    config: Config,
    metadata: OnceLock<Metadata>,
}

impl Context {
    pub fn load() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .context("xtask must be located directly below the workspace root")?
            .to_path_buf();
        let config = Config::load(&root.join(".xtask.toml"))?;
        Ok(Self {
            root,
            config,
            metadata: OnceLock::new(),
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn metadata(&self) -> Result<&Metadata> {
        if let Some(metadata) = self.metadata.get() {
            return Ok(metadata);
        }

        let metadata = MetadataCommand::new()
            .current_dir(&self.root)
            .no_deps()
            .exec()
            .with_context(|| {
                format!("failed to read Cargo metadata from {}", self.root.display())
            })?;
        let _ = self.metadata.set(metadata);
        self.metadata
            .get()
            .ok_or_else(|| anyhow::anyhow!("Cargo metadata cache was not initialized"))
    }

    pub fn workspace_version(&self) -> Result<Version> {
        let packages = self.metadata()?.workspace_packages();
        let first = packages
            .first()
            .ok_or_else(|| anyhow::anyhow!("Cargo metadata contains no workspace packages"))?;
        let version = first.version.clone();
        if packages.iter().any(|package| package.version != version) {
            anyhow::bail!("workspace packages do not share a common version")
        }
        Ok(version)
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.root.join(relative)
    }

    pub fn python(&self, args: &[&str]) -> Result<()> {
        let candidates = self.python_candidates();
        let mut not_found = Vec::new();

        for candidate in &candidates {
            match self.try_run(candidate, args) {
                Ok(status) if status.success() => return Ok(()),
                Ok(status) => {
                    anyhow::bail!("{candidate} exited with status {status}")
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    not_found.push(candidate.as_str());
                }
                Err(error) => {
                    return Err(error.into());
                }
            }
        }

        anyhow::bail!(
            "could not find Python executable; tried: {}",
            not_found.join(", ")
        )
    }

    pub fn python_output(&self, args: &[&str]) -> Result<String> {
        let candidates = self.python_candidates();
        let mut not_found = Vec::new();

        for candidate in &candidates {
            match self.try_output(candidate, args) {
                Ok(output) if output.status.success() => {
                    return String::from_utf8(output.stdout)
                        .context("Python returned non-UTF-8 output");
                }
                Ok(output) => {
                    anyhow::bail!("{candidate} exited with status {}", output.status)
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    not_found.push(candidate.as_str());
                }
                Err(error) => {
                    return Err(error.into());
                }
            }
        }

        anyhow::bail!(
            "could not find Python executable; tried: {}",
            not_found.join(", ")
        )
    }

    pub fn cargo(&self, args: &[&str]) -> Result<()> {
        self.run("cargo", args)
    }

    pub fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        self.try_run(program, args)
            .with_context(|| format!("failed to start {program}"))
            .and_then(|status| {
                if status.success() {
                    Ok(())
                } else {
                    anyhow::bail!("{program} exited with status {status}")
                }
            })
    }

    pub fn output(&self, program: &str, args: &[&str]) -> Result<std::process::Output> {
        Command::new(program)
            .args(args)
            .current_dir(&self.root)
            .output()
            .with_context(|| format!("failed to start {program}"))
    }

    pub fn command_available(&self, program: &str) -> bool {
        Command::new(program)
            .arg("--version")
            .current_dir(&self.root)
            .output()
            .is_ok_and(|output| output.status.success())
    }

    pub fn python_command(&self) -> String {
        self.python_candidates()
            .into_iter()
            .next()
            .unwrap_or_else(|| "python".to_owned())
    }

    fn python_candidates(&self) -> Vec<String> {
        if cfg!(windows) {
            vec!["python".to_owned(), "python3".to_owned()]
        } else {
            vec!["python3".to_owned(), "python".to_owned()]
        }
    }

    fn try_run(&self, program: &str, args: &[&str]) -> std::io::Result<std::process::ExitStatus> {
        Command::new(program)
            .args(args)
            .current_dir(&self.root)
            .status()
    }

    fn try_output(&self, program: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
        Command::new(program)
            .args(args)
            .current_dir(&self.root)
            .output()
    }
}
