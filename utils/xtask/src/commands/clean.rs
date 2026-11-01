use crate::context::Context;
use anyhow::{Context as _, Result};
use std::{fs, path::Path};

const ROOT_ARTIFACT_DIRECTORIES: &[&str] = &["target", "dist", "build"];
const PYTHON_CACHE_DIRECTORIES: &[&str] = &[
    "__pycache__",
    ".pytest_cache",
    ".ruff_cache",
    ".mypy_cache",
    "htmlcov",
];

pub(super) fn run(context: &Context) -> Result<()> {
    for directory in ROOT_ARTIFACT_DIRECTORIES {
        remove_directory(context.path(directory))?;
    }

    clean_python_artifacts(context.root())?;
    crate::success!(
        "cleaned Rust and Python artifacts; preserved .venv, lockfiles, source, data, and models"
    );
    Ok(())
}

fn remove_directory(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("failed to remove {}", path.display()))?;
        crate::info!("removed {}", path.display());
    }
    Ok(())
}

fn clean_python_artifacts(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if PYTHON_CACHE_DIRECTORIES.contains(&name.as_ref()) || name.ends_with(".egg-info") {
                remove_directory(&entry_path)?;
                continue;
            }

            if name == ".git" || name == ".venv" || name == "target" {
                continue;
            }

            clean_python_artifacts(&entry_path)?;
        } else if file_type.is_file()
            && matches!(
                entry_path
                    .extension()
                    .and_then(|extension| extension.to_str()),
                Some("pyc" | "pyo")
            )
        {
            fs::remove_file(&entry_path)
                .with_context(|| format!("failed to remove {}", entry_path.display()))?;
            crate::info!("removed {}", entry_path.display());
        }
    }
    Ok(())
}
