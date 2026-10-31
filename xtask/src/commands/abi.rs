use crate::context::Context;
use anyhow::{Result, bail};

const ABI_PROBE: &str = r#"
import sys
import sysconfig

print(f"implementation={sys.implementation.name}")
print(f"version={sys.version_info.major}.{sys.version_info.minor}")
print(f"soabi={sysconfig.get_config_var('SOABI') or ''}")
print(f"extension_suffix={sysconfig.get_config_var('EXT_SUFFIX') or ''}")

try:
    import torch
except ImportError:
    print("pytorch=")
else:
    print(f"pytorch={torch.__version__}")
"#;

pub(super) fn run(context: &Context) -> Result<()> {
    let output = context.python_output(&["-c", ABI_PROBE])?;
    let probe = AbiProbe::parse(&output)?;
    let python = &context.config().python;
    let mut failures = Vec::new();

    crate::info!("runtime implementation: {}", probe.implementation);
    crate::info!("runtime Python version: {}", probe.version);
    crate::info!(
        "runtime SOABI: {}",
        if probe.soabi.is_empty() {
            "n/a"
        } else {
            &probe.soabi
        }
    );
    crate::info!(
        "runtime extension suffix: {}",
        if probe.extension_suffix.is_empty() {
            "n/a"
        } else {
            &probe.extension_suffix
        }
    );

    if probe.implementation != python.implementation {
        failures.push(format!(
            "implementation mismatch: configured {}, found {}",
            python.implementation, probe.implementation
        ));
    }

    if probe.version != python.version {
        failures.push(format!(
            "Python version mismatch: configured {}, found {}",
            python.version, probe.version
        ));
    }

    if !abi_matches(&python.abi, &probe.soabi, &probe.extension_suffix) {
        failures.push(format!(
            "ABI mismatch: configured {}, SOABI `{}`, extension suffix `{}`",
            python.abi, probe.soabi, probe.extension_suffix
        ));
    }

    if let Some(expected) = python.pytorch.as_deref() {
        if let Some(actual) = probe.pytorch.as_deref() {
            if !actual.starts_with(expected) {
                failures.push(format!(
                    "PyTorch mismatch: configured {expected}, found {actual}"
                ));
            }
        } else {
            failures.push(format!(
                "PyTorch {expected} is configured but not installed"
            ));
        }
    }

    if failures.is_empty() {
        crate::success!("Python ABI compatibility check passed");
        Ok(())
    } else {
        for failure in &failures {
            crate::error!("{failure}");
        }
        bail!("Python ABI compatibility check failed")
    }
}

#[derive(Debug)]
struct AbiProbe {
    implementation: String,
    version: String,
    soabi: String,
    extension_suffix: String,
    pytorch: Option<String>,
}

impl AbiProbe {
    fn parse(output: &str) -> Result<Self> {
        let mut implementation = None;
        let mut version = None;
        let mut soabi = None;
        let mut extension_suffix = None;
        let mut pytorch = None;

        for line in output.lines() {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("invalid Python ABI probe output: {line}"))?;
            match key {
                "implementation" => implementation = Some(value.to_owned()),
                "version" => version = Some(value.to_owned()),
                "soabi" => soabi = Some(value.to_owned()),
                "extension_suffix" => extension_suffix = Some(value.to_owned()),
                "pytorch" => pytorch = (!value.is_empty()).then(|| value.to_owned()),
                other => bail!("Python ABI probe returned unknown field `{other}`"),
            }
        }

        Ok(Self {
            implementation: required(implementation, "implementation")?,
            version: required(version, "version")?,
            soabi: required(soabi, "soabi")?,
            extension_suffix: required(extension_suffix, "extension_suffix")?,
            pytorch,
        })
    }
}

fn required(value: Option<String>, key: &str) -> Result<String> {
    value.ok_or_else(|| anyhow::anyhow!("Python ABI probe did not return `{key}`"))
}

fn abi_matches(expected: &str, soabi: &str, extension_suffix: &str) -> bool {
    if soabi.contains(expected) || extension_suffix.contains(expected) {
        return true;
    }

    expected
        .strip_prefix("cp")
        .map(|version| soabi.contains(&format!("cpython-{version}")))
        .unwrap_or(false)
}
