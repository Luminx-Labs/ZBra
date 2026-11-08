use crate::{cli::VersionBump, context::Context};
use anyhow::{Context as _, Result, bail};
use semver::{BuildMetadata, Prerelease, Version};
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml_edit::{DocumentMut, value};

pub(super) fn run(
    context: &Context,
    requested: Option<&Version>,
    bump: Option<VersionBump>,
    pre_release: bool,
) -> Result<()> {
    let current = current(context)?;
    let next = match (requested, bump) {
        (Some(_), Some(_)) => bail!("--set and --bump cannot be used together"),
        (Some(version), None) => Some(version.clone()),
        (None, Some(bump)) => Some(next_version(&current, bump, pre_release)?),
        (None, None) => None,
    };
    let manifest = context.path("Cargo.toml");
    let text = fs::read_to_string(&manifest)
        .with_context(|| format!("failed to read {}", manifest.display()))?;
    let mut document = text
        .parse::<DocumentMut>()
        .with_context(|| format!("failed to parse {}", manifest.display()))?;
    match next.as_ref() {
        None => crate::info!("workspace version: {current}"),
        Some(next) => {
            document["workspace"]["package"]["version"] = value(next.to_string());
            fs::write(&manifest, document.to_string())
                .with_context(|| format!("failed to write {}", manifest.display()))?;
            for path in python_versions(context)? {
                update_python_version(&path, next)?;
            }
            update_nnue_version(&context.path("nnue/__init__.py"), next)?;
            crate::info!("updating Cargo.lock");
            context.cargo(&["check", "--workspace"])?;
            crate::success!("updated workspace version: {current} -> {next}");
        }
    }
    Ok(())
}

pub(super) fn current(context: &Context) -> Result<Version> {
    context.workspace_version()
}

fn next_version(current: &Version, bump: VersionBump, pre_release: bool) -> Result<Version> {
    let (major, minor, patch) = match bump {
        VersionBump::Major => (
            current
                .major
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("major version overflow"))?,
            0,
            0,
        ),
        VersionBump::Minor => (
            current.major,
            current
                .minor
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("minor version overflow"))?,
            0,
        ),
        VersionBump::Patch => (
            current.major,
            current.minor,
            current
                .patch
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("patch version overflow"))?,
        ),
    };

    Ok(Version {
        major,
        minor,
        patch,
        pre: if pre_release {
            Prerelease::new("alpha.1")?
        } else {
            Prerelease::EMPTY
        },
        build: BuildMetadata::EMPTY,
    })
}

fn python_versions(context: &Context) -> Result<Vec<PathBuf>> {
    fn visit(dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path
                .file_name()
                .is_some_and(|name| name == "target" || name == ".git")
            {
                continue;
            }
            if path.is_dir() {
                visit(&path, paths)?;
            } else if path
                .file_name()
                .is_some_and(|name| name == "pyproject.toml")
            {
                paths.push(path);
            }
        }
        Ok(())
    }
    let mut paths = Vec::new();
    visit(context.root(), &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn update_python_version(path: &Path, version: &Version) -> Result<()> {
    let text = fs::read_to_string(path)?;
    let mut document = text.parse::<DocumentMut>()?;
    document["project"]["version"] = value(version.to_string());
    fs::write(path, document.to_string())?;
    Ok(())
}

fn update_nnue_version(path: &Path, version: &Version) -> Result<()> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut matches = 0;
    let updated = text
        .split_inclusive('\n')
        .map(|line| {
            if line.trim_start().starts_with("__version__") {
                matches += 1;
                let newline = if line.ends_with('\n') { "\n" } else { "" };
                let indentation = &line[..line.len() - line.trim_start().len()];
                format!("{indentation}__version__ = \"{version}\"{newline}")
            } else {
                line.to_owned()
            }
        })
        .collect::<String>();
    anyhow::ensure!(
        matches == 1,
        "expected exactly one __version__ assignment in {}",
        path.display()
    );
    fs::write(path, updated).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{next_version, update_nnue_version};
    use crate::cli::VersionBump;
    use semver::Version;
    use std::fs;

    #[test]
    fn bumps_semver_components_and_clears_metadata() {
        let current = Version::parse("1.2.3-alpha+build.7").unwrap();
        assert_eq!(
            next_version(&current, VersionBump::Major, false).unwrap(),
            Version::parse("2.0.0").unwrap()
        );
        assert_eq!(
            next_version(&current, VersionBump::Minor, false).unwrap(),
            Version::parse("1.3.0").unwrap()
        );
        assert_eq!(
            next_version(&current, VersionBump::Patch, false).unwrap(),
            Version::parse("1.2.4").unwrap()
        );
    }

    #[test]
    fn can_mark_a_bump_as_an_alpha_prerelease() {
        let current = Version::parse("1.2.3").unwrap();
        assert_eq!(
            next_version(&current, VersionBump::Patch, true).unwrap(),
            Version::parse("1.2.4-alpha.1").unwrap()
        );
    }

    #[test]
    fn updates_nnue_package_version() {
        let path = std::env::temp_dir().join(format!(
            "zbra-xtask-version-test-{}-{}.py",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(
            &path,
            "__all__ = [\"__version__\"]\n__version__ = \"0.1.0\"\n",
        )
        .unwrap();

        update_nnue_version(&path, &Version::parse("0.2.0").unwrap()).unwrap();

        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "__all__ = [\"__version__\"]\n__version__ = \"0.2.0\"\n"
        );
        fs::remove_file(path).unwrap();
    }
}
