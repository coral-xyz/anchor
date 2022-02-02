use anyhow::{anyhow, Result};
use dialoguer::Input;
use once_cell::sync::Lazy;
use reqwest::header::USER_AGENT;
use semver::Version;
use serde::{de, Deserialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Stdio;

/// Storage directory for AVM, ~/.avm
pub static AVM_HOME: Lazy<PathBuf> = Lazy::new(|| {
    cfg_if::cfg_if! {
        if #[cfg(test)] {
            let dir = tempfile::tempdir().expect("Could not create temporary directory");
            dir.path().join(".avm")
        } else {
            let mut user_home = dirs::home_dir().expect("Could not find home directory");
            user_home.push(".avm");
            user_home
        }
    }
});

/// Path to the current version file ~/.avm/.version
pub fn current_version_path() -> PathBuf {
    let mut current_version_path = AVM_HOME.to_path_buf();
    current_version_path.push(".version");
    current_version_path
}

/// Read the current version from the version file
pub fn current_version() -> Result<Version> {
    let v = fs::read_to_string(current_version_path().as_path())
        .map_err(|e| anyhow!("Could not read version file: {}", e))?;
    Ok(Version::parse(v.trim_end_matches('\n').to_string().as_str()).unwrap())
}

/// Path to the binary for the current version
pub fn current_version_binary_path(version: &Version) -> PathBuf {
    let mut version_path = AVM_HOME.join("bin").to_path_buf();
    version_path.push(format!("anchor-{}", version));
    version_path
}

/// Update the current version to a new version
pub fn use_version(version: &Version) -> Result<()> {
    let installed_versions = read_installed_versions();
    // Make sure the requested version is installed
    if !installed_versions.contains(version) {
        let input: String = Input::new()
            .with_prompt(format!(
                "anchor-cli {} is not installed, would you like to install it? (y/n)",
                version.to_string()
            ))
            .with_initial_text("y")
            .default("n".into())
            .interact_text()?;
        if matches!(input.as_str(), "y" | "yy" | "Y" | "yes" | "Yes") {
            install_version(version)?;
        }
    }

    let mut current_version_file = fs::File::create(current_version_path().as_path())?;
    current_version_file.write_all(version.to_string().as_bytes())?;
    Ok(())
}

/// Install a version of anchor-cli
pub fn install_version(version: &Version) -> Result<()> {
    let exit = std::process::Command::new("cargo")
        .args(&[
            "install",
            "--git",
            "https://github.com/project-serum/anchor",
            "--tag",
            &format!("v{}", &version),
            "anchor-cli",
            "--locked",
            "--root",
            &AVM_HOME.to_str().unwrap(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| {
            anyhow::format_err!("Cargo install for {} failed: {}", version, e.to_string())
        })?;
    if !exit.status.success() {
        return Err(anyhow!(
            "Failed to install {}, is it a valid version?",
            version
        ));
    }
    fs::rename(
        &AVM_HOME.join("bin").join("anchor"),
        &AVM_HOME
            .join("bin")
            .join(format!("anchor-{}", version.to_string())),
    )?;
    Ok(())
}

/// Remove an installed version of anchor-cli
pub fn uninstall_version(version: &Version) -> Result<()> {
    let version_path = AVM_HOME
        .join("bin")
        .join(format!("anchor-{}", version.to_string()));
    if !version_path.exists() {
        return Err(anyhow!("Anchor CLI {} is not installed", version));
    }
    fs::remove_file(version_path.as_path())?;
    Ok(())
}

/// Ensure the users home directory is setup with the paths required by AVM.
pub fn ensure_paths() {
    let home_dir = AVM_HOME.to_path_buf();
    if !home_dir.as_path().exists() {
        fs::create_dir_all(home_dir.clone()).expect("Could not create .avm directory");
    }
    let bin_dir = home_dir.join("bin");
    if !bin_dir.as_path().exists() {
        fs::create_dir_all(bin_dir.clone()).expect("Could not create .avm/bin directory");
    }
    if !current_version_path().exists() {
        fs::File::create(current_version_path()).expect("Could not create .version file");
    }
}

/// Retrieve a list of installable versions of anchor-cli using the GitHub API and tags on the Anchor
/// repository.
pub fn fetch_versions() -> Vec<semver::Version> {
    #[derive(Deserialize)]
    struct Release {
        #[serde(rename = "name", deserialize_with = "version_deserializer")]
        version: semver::Version,
    }

    fn version_deserializer<'de, D>(deserializer: D) -> Result<semver::Version, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let s: &str = de::Deserialize::deserialize(deserializer)?;
        Version::parse(s.trim_start_matches("v")).map_err(de::Error::custom)
    }

    let client = reqwest::blocking::Client::new();
    let versions: Vec<Release> = client
        .get("https://api.github.com/repos/project-serum/anchor/tags")
        .header(USER_AGENT, "avm https://github.com/project-serum/anchor")
        .send()
        .unwrap()
        .json()
        .unwrap();
    versions.into_iter().map(|r| r.version).collect()
}

/// Print available versions and flags indicating installed, current and latest
pub fn list_versions() -> Result<()> {
    let installed_versions = read_installed_versions();

    let mut available_versions = fetch_versions();
    // Reverse version list so latest versions are printed last
    available_versions.reverse();

    available_versions.iter().enumerate().for_each(|(i, v)| {
        print!("{}", v);
        let mut flags = vec![];
        if i == available_versions.len() - 1 {
            flags.push("latest");
        }
        if installed_versions.contains(&v) {
            flags.push("installed");
        }
        if current_version().unwrap() == v.clone() {
            flags.push("current");
        }
        if flags.is_empty() {
            println!();
        } else {
            println!("\t({})", flags.join(", "));
        }
    });

    Ok(())
}

pub fn get_latest_version() -> semver::Version {
    let available_versions = fetch_versions();
    available_versions.first().unwrap().clone()
}

/// Read the installed anchor-cli versions by reading the binaries in the AVM_HOME/bin directory.
pub fn read_installed_versions() -> Vec<semver::Version> {
    let home_dir = AVM_HOME.to_path_buf();
    let mut versions = vec![];
    for file in fs::read_dir(&home_dir.join("bin")).unwrap() {
        versions.push(
            Version::parse(
                file.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .trim_start_matches("anchor-"),
            )
            .expect("Failed to parse version"),
        );
    }

    versions
}
