//! Manage the standalone SBF builder independently of the selected Solana CLI.
use {
    crate::{AVM_HOME, HTTP_CLIENT},
    anyhow::{bail, Context, Result},
    chrono::Utc,
    fs2::FileExt,
    semver::Version,
    serde::{Deserialize, Serialize},
    std::{fs, path::Path, process::Command},
};

const INDEX_URL: &str = "https://index.crates.io/ca/rg/cargo-build-sbf";
const CHECK_INTERVAL: i64 = 24 * 60 * 60;
// Separate from the old .cargo-build-sbf-check, which cached platform-tools tags.
const CACHE_FILE: &str = ".build-sbf-check.json";

#[derive(Deserialize)]
struct Release {
    vers: String,
    yanked: bool,
}

#[derive(Deserialize, Serialize)]
struct Check {
    timestamp: i64,
    version: String,
}

fn latest_stable(index: &str) -> Result<Version> {
    let mut latest = None;
    for line in index.lines().filter(|line| !line.trim().is_empty()) {
        let release: Release =
            serde_json::from_str(line).context("Parsing cargo-build-sbf index")?;
        let version = Version::parse(&release.vers)?;
        if !release.yanked && version.pre.is_empty() && latest.as_ref().is_none_or(|v| &version > v)
        {
            latest = Some(version);
        }
    }
    latest.context("No stable cargo-build-sbf release in crates.io index")
}

fn fetch_latest() -> Result<Version> {
    let index = HTTP_CLIENT
        .get(INDEX_URL)
        .header(
            reqwest::header::USER_AGENT,
            "avm https://github.com/otter-sec/anchor",
        )
        .send()?
        .error_for_status()?
        .text()?;
    latest_stable(&index)
}

fn installed_version(home: &Path) -> Option<Version> {
    let output = Command::new(
        home.join("bin")
            .join(format!("cargo-build-sbf{}", std::env::consts::EXE_SUFFIX)),
    )
    .arg("--version")
    .output()
    .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut fields = stdout.lines().next()?.split_whitespace();
    if fields.next()? != "cargo-build-sbf" {
        return None;
    }
    Version::parse(fields.next()?).ok()
}

fn install(home: &Path, version: &Version) -> Result<()> {
    eprintln!(
        "Installing cargo-build-sbf {version} into {}",
        home.display()
    );
    let status = Command::new("cargo")
        // Do not let the program's rust-toolchain.toml choose the installer compiler.
        .current_dir(home)
        .args([
            "install",
            "cargo-build-sbf",
            "--locked",
            "--registry",
            "crates-io",
            "--version",
        ])
        .arg(format!("={version}"))
        .arg("--root")
        .arg(home)
        .arg("--force")
        .status()
        .context("Running cargo install for AVM's cargo-build-sbf")?;
    if !status.success() {
        bail!(
            "Installing cargo-build-sbf {version} failed ({status}); check the Cargo/Rust \
             installation above"
        );
    }
    Ok(())
}

/// Ensure an AVM-owned builder before launching Anchor. Checks are cached for
/// one day. An existing managed installation remains usable offline; a missing
/// installation is an error, never a fallback to Solana's bundled builder.
pub fn ensure_cargo_build_sbf() -> Result<()> {
    fs::create_dir_all(&*AVM_HOME)?;
    let home = fs::canonicalize(&*AVM_HOME)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(home.join(".build-sbf.lock"))?;
    lock.lock_exclusive()?;
    ensure_with(&home, Utc::now().timestamp(), fetch_latest, install)
}

fn ensure_with(
    home: &Path,
    now: i64,
    fetch: impl FnOnce() -> Result<Version>,
    install: impl FnOnce(&Path, &Version) -> Result<()>,
) -> Result<()> {
    let installed = installed_version(home);
    let cached = fs::read(home.join(CACHE_FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Check>(&bytes).ok())
        .filter(|check| (0..CHECK_INTERVAL).contains(&(now - check.timestamp)))
        .and_then(|check| {
            Version::parse(&check.version)
                .ok()
                .map(|version| (check.timestamp, version))
        })
        .filter(|(_, version)| version.pre.is_empty());
    let mut checked_at = cached.as_ref().map_or(now, |(timestamp, _)| *timestamp);
    let desired = match cached.map(|(_, version)| Ok(version)).unwrap_or_else(fetch) {
        Ok(version) => version,
        Err(error) => match &installed {
            Some(version) => {
                eprintln!(
                    "Could not check cargo-build-sbf updates: {error:#}; using AVM's cached \
                     {version}"
                );
                version.clone()
            }
            None => {
                return Err(error.context("Resolving cargo-build-sbf for AVM's first installation"))
            }
        },
    };
    let mut checked_version = desired.clone();
    if installed.as_ref().is_none_or(|version| version < &desired) {
        let result = install(home, &desired).and_then(|()| {
            if installed_version(home)
                .as_ref()
                .is_none_or(|version| version < &desired)
            {
                bail!(
                    "cargo install did not produce cargo-build-sbf {desired} in {}/bin",
                    home.display()
                );
            }
            Ok(())
        });
        if let Err(error) = result {
            // Recheck the executable: a failed install may have removed it.
            match installed_version(home) {
                Some(version) => {
                    eprintln!(
                        "Could not update cargo-build-sbf: {error:#}; using AVM's cached {version}"
                    );
                    checked_version = version;
                    checked_at = now;
                }
                None => return Err(error),
            }
        }
    }
    fs::write(
        home.join(CACHE_FILE),
        serde_json::to_vec(&Check {
            timestamp: checked_at,
            version: checked_version.to_string(),
        })?,
    )?;
    Ok(())
}
