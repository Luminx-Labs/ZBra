use crate::{config::UtilityLanguage, context::Context};
use anyhow::{Context as _, Result, bail};

pub(super) fn list(context: &Context) -> Result<()> {
    if context.config().utils.is_empty() {
        crate::warn!("no native utilities are configured");
        return Ok(());
    }

    for utility in &context.config().utils {
        crate::info!(
            "{} [{}] -> {} (optional: {}; {})",
            utility.name,
            language_name(&utility.language),
            context.path(&utility.binary).display(),
            utility.optional,
            utility.purpose
        );
    }
    Ok(())
}

pub(super) fn build(context: &Context) -> Result<()> {
    for utility in &context.config().utils {
        let source = context.path(&utility.path);
        let source_exists = match utility.language {
            UtilityLanguage::Rust => source.join("Cargo.toml").is_file(),
            UtilityLanguage::C => source.is_file(),
        };
        if !source_exists {
            if utility.optional {
                crate::warn!(
                    "skipping optional utility {}: source {} does not exist",
                    utility.name,
                    source.display()
                );
                continue;
            }
            bail!(
                "utility {} source does not exist: {}",
                utility.name,
                source.display()
            );
        }

        let binary = context.path(&utility.binary);
        if let Some(parent) = binary.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        match utility.language {
            UtilityLanguage::Rust => {
                let manifest = context.path(&utility.path).join("Cargo.toml");
                context.cargo(&[
                    "build",
                    "--release",
                    "--manifest-path",
                    manifest
                        .to_str()
                        .context("utility manifest is not valid UTF-8")?,
                ])?;
            }
            UtilityLanguage::C => {
                let compiler = if cfg!(windows) { "clang" } else { "cc" };
                context.run(
                    compiler,
                    &[
                        "-O3",
                        source
                            .to_str()
                            .context("utility source is not valid UTF-8")?,
                        "-o",
                        binary
                            .to_str()
                            .context("utility binary is not valid UTF-8")?,
                    ],
                )?;
            }
        }
        crate::success!("built utility {}", utility.name);
    }
    Ok(())
}

fn language_name(language: &UtilityLanguage) -> &'static str {
    match language {
        UtilityLanguage::Rust => "rust",
        UtilityLanguage::C => "c",
    }
}
