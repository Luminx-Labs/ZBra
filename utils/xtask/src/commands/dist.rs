use crate::{commands::version, context::Context};
use anyhow::{Context as _, Result};
use std::fs;

const ARCHIVE_SCRIPT: &str = r#"
import hashlib
import pathlib
import sys
import zipfile

root = pathlib.Path.cwd()
archive = pathlib.Path(sys.argv[1])
version = sys.argv[2]
files = [pathlib.Path(path) for path in sys.argv[3:]]
files = [path for path in files if (root / path).is_file()]

with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
    for path in files:
        output.write(root / path, f"zbra-{version}/{path.as_posix()}")

digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix(archive.suffix + ".sha256").write_text(
    f"{digest}  {archive.name}\n", encoding="utf-8"
)
"#;

pub(super) fn run(context: &Context) -> Result<()> {
    super::build::run(context)?;
    let version = version::current(context)?;
    let dist_dir = context.path("dist");
    fs::create_dir_all(&dist_dir)
        .with_context(|| format!("failed to create {}", dist_dir.display()))?;
    let archive = dist_dir.join(format!(
        "{}-v{}-{}-{}.zip",
        context.config().project.name,
        version,
        std::env::consts::OS,
        std::env::consts::ARCH
    ));
    let mut assets = vec![
        "README.md".to_owned(),
        ".xtask.toml".to_owned(),
        context.config().paths.engine_binary.clone(),
        context.config().model.default.clone(),
    ];
    assets.extend(
        context
            .config()
            .utils
            .iter()
            .map(|utility| utility.binary.clone()),
    );

    let archive_text = archive
        .to_str()
        .context("archive path is not valid UTF-8")?;
    let version_text = version.to_string();
    let mut python_args = vec!["-c", ARCHIVE_SCRIPT, archive_text, version_text.as_str()];
    python_args.extend(assets.iter().map(String::as_str));
    context.python(&python_args)?;
    crate::success!("created {}", archive.display());
    crate::success!("created {}.sha256", archive.display());
    Ok(())
}
