use crate::{cli::VersionBump, context::Context};
use anyhow::{Context as _, Result, bail};
use semver::{BuildMetadata, Prerelease, Version};
use std::fs;
use toml_edit::{DocumentMut, value};

pub(super) fn run(
    context: &Context,
    requested: Option<&Version>,
    bump: Option<VersionBump>,
) -> Result<()> {
    let current = current(context)?;
    let next = match (requested, bump) {
        (Some(_), Some(_)) => bail!("--set and --bump cannot be used together"),
        (Some(version), None) => Some(version.clone()),
        (None, Some(bump)) => Some(next_version(&current, bump)?),
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
            if next == &current {
                crate::info!("workspace version is already {current}");
                return Ok(());
            }
            document["workspace"]["package"]["version"] = value(next.to_string());
            fs::write(&manifest, document.to_string())
                .with_context(|| format!("failed to write {}", manifest.display()))?;
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

fn next_version(current: &Version, bump: VersionBump) -> Result<Version> {
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
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY,
    })
}

#[cfg(test)]
mod tests {
    use super::next_version;
    use crate::cli::VersionBump;
    use semver::Version;

    #[test]
    fn bumps_semver_components_and_clears_metadata() {
        let current = Version::parse("1.2.3-alpha+build.7").unwrap();
        assert_eq!(
            next_version(&current, VersionBump::Major).unwrap(),
            Version::parse("2.0.0").unwrap()
        );
        assert_eq!(
            next_version(&current, VersionBump::Minor).unwrap(),
            Version::parse("1.3.0").unwrap()
        );
        assert_eq!(
            next_version(&current, VersionBump::Patch).unwrap(),
            Version::parse("1.2.4").unwrap()
        );
    }
}
