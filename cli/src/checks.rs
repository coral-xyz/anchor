use std::{fs, path::Path};
use anyhow::{anyhow, Context, Result};
use semver::{Version, VersionReq};
use crate::{
    config::{Config, Manifest, WithPath},
    VERSION,
};

/// Check whether `overflow-checks` codegen option is enabled.
///
/// https://doc.rust-lang.org/rustc/codegen-options/index.html#overflow-checks
pub fn check_overflow(cargo_toml_path: impl AsRef<Path>) -> Result<bool> {
    let manifest = Manifest::from_path(cargo_toml_path.as_ref())
        .context("Failed to parse Cargo.toml for overflow-checks")?;
    
    manifest.profile
        .release
        .as_ref()
        .and_then(|profile| profile.overflow_checks)
        .ok_or_else(|| anyhow!(
            "`overflow-checks` is not enabled. To enable, add the following to the root Cargo.toml:\n\n\
            [profile.release]\n\
            overflow-checks = true\n"
        ))
}

/// Check whether there is a mismatch between the current CLI version and:
///
/// - `anchor-lang` crate version
/// - `@coral-xyz/anchor` package version
///
/// This function logs warnings in the case of a mismatch.
pub fn check_anchor_version(cfg: &WithPath<Config>) -> Result<()> {
    let cli_version = Version::parse(VERSION)
        .context("Failed to parse CLI version")?;

    // Check lang crate
    if let Some(ver) = cfg.get_rust_program_list()?
        .into_iter()
        .map(|path| path.join("Cargo.toml"))
        .filter_map(|path| Manifest::from_path(&path).ok())
        .filter_map(|man| man.dependencies.get("anchor-lang").cloned())
        .filter_map(|dep| Version::parse(dep.req()).ok())
        .find(|ver| ver != &cli_version)
    {
        eprintln!(
            "WARNING: `anchor-lang` version ({ver}) does not match the current CLI version \
            ({cli_version}). This may lead to unexpected behavior.\n\n\
            To align the versions, add the following to Anchor.toml:\n\n\
            [toolchain]\n\
            anchor_version = \"{ver}\"\n"
        );
    }

    // Check TS package
    let package_json_path = cfg.path().parent().unwrap().join("package.json");
    let package_json_content = fs::read_to_string(&package_json_path)
        .context("Failed to read package.json for version check")?;
    
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)
        .context("Failed to parse package.json for version check")?;
    
    if let Some(ver) = package_json
        .get("dependencies")
        .and_then(|deps| deps.get("@coral-xyz/anchor"))
        .and_then(|ver| ver.as_str())
        .and_then(|ver| VersionReq::parse(ver).ok())
        .filter(|ver| !ver.matches(&cli_version))
    {
        eprintln!(
            "WARNING: `@coral-xyz/anchor` version ({ver}) does not match the current CLI version \
            ({cli_version}). This may lead to unexpected behavior.\n\n\
            To fix this issue, upgrade the package by running:\n\n\
            yarn upgrade @coral-xyz/anchor@{cli_version}\n"
        );
    }

    Ok(())
}

/// Check whether the `idl-build` feature is being used correctly.
///
/// **Note:** The check expects the current directory to be a program directory.
pub fn check_idl_build_feature() -> Result<()> {
    let manifest = Manifest::from_path("Cargo.toml")
        .context("Failed to parse Cargo.toml for IDL build feature check")?;

    // Check if `idl-build` is enabled by default
    manifest.dependencies.iter()
        .filter(|(_, dep)| dep.req_features().contains(&"idl-build".into()))
        .for_each(|(name, _)| {
            eprintln!(
                "WARNING: The `idl-build` feature for crate `{name}` is enabled by default, which \
                is not the intended usage.\n\n\
                To resolve this, do not enable the `idl-build` feature by default. Instead, include \
                crates that require `idl-build` in the `idl-build` feature list:\n\n\
                [features]\n\
                idl-build = [\"{name}/idl-build\", ...]\n"
            );
        });

    // Check `anchor-spl`'s `idl-build` feature
    if manifest.dependencies.get("anchor-spl").is_some() &&
        manifest.features.get("idl-build")
            .map(|feature_list| !feature_list.contains(&"anchor-spl/idl-build".into()))
            .unwrap_or(true)
    {
        eprintln!(
            "WARNING: The `idl-build` feature of `anchor-spl` is not enabled, which may lead to \
            cryptic compile errors.\n\n\
            To resolve this, add `anchor-spl/idl-build` to the `idl-build` feature list:\n\n\
            [features]\n\
            idl-build = [\"anchor-spl/idl-build\", ...]\n"
        );
    }

    Ok(())
}
