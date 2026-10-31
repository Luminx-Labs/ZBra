use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub project: ProjectConfig,
    #[serde(default)]
    pub paths: PathsConfig,
    #[serde(default)]
    pub python: PythonConfig,
    #[serde(default)]
    pub model: ModelConfig,
    #[serde(default)]
    pub environment: EnvironmentConfig,
    #[serde(default)]
    pub libraries: Vec<LibraryConfig>,
    #[serde(default)]
    pub models: Vec<TrackedModelConfig>,
    #[serde(default)]
    pub utils: Vec<UtilityConfig>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        toml_edit::de::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display()))
    }
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "zbra".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PathsConfig {
    pub python_dir: String,
    pub model_dir: String,
    pub data_dir: String,
    pub engine_binary: String,
    pub hooks_dir: String,
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
pub struct PythonConfig {
    pub version: String,
    pub abi: String,
    pub implementation: String,
    pub package_manager: String,
    pub project_dir: String,
    pub virtual_environment: String,
    pub lockfile: String,
    pub pytorch: Option<String>,
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
            pytorch: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub default: String,
    pub format_version: u32,
    pub require_checksum: bool,
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
pub struct EnvironmentConfig {
    pub rust: String,
    pub cuda: Option<String>,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            rust: "1.88".to_owned(),
            cuda: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LibraryConfig {
    pub name: String,
    pub ecosystem: String,
    pub version: String,
    pub purpose: String,
    pub optional: bool,
    pub python_abi: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackedModelConfig {
    pub name: String,
    pub path: String,
    pub version: String,
    pub format_version: u32,
    pub architecture: String,
    pub status: String,
    pub sha256: Option<String>,
    pub dataset: Option<String>,
    pub training_commit: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UtilityConfig {
    pub name: String,
    pub language: UtilityLanguage,
    pub path: String,
    pub binary: String,
    pub purpose: String,
    #[serde(default)]
    pub optional: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UtilityLanguage {
    Rust,
    C,
}
