use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::Result;
use cargo_semver_checks::{Check, Rustdoc};
use clap::{Args, Parser};
use git_cliff_core::{commit::Commit, release::Release, repo::Repository};
use toml_edit::Document;

type ReleaseType = cargo_semver_checks::ReleaseType;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(version, propagate_version = true)]
enum Cargo {
    SemverBump(SemverBump),
}

#[derive(Args)]
#[command()]
struct SemverBump {
    /// Run without making any changes
    #[arg(short, long, default_value = "false")]
    dry_run: bool,

    /// Path to project
    #[arg(short, long, default_value = "./")]
    path: PathBuf,

    /// Enable verbose output
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

fn main() -> Result<()> {
    let Cargo::SemverBump(args) = Cargo::parse();

    let verbosity = if args.verbose { "debug" } else { "info" };

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(verbosity)).init();

    log::debug!("Reading Cargo.toml");

    let cargo_toml_path = args.path.join("Cargo.toml");

    let mut cargo_toml_contents = String::new();
    {
        let mut raw_cargo_toml =
            File::open(&cargo_toml_path).unwrap_or_else(|_| log_panic("Could not open Cargo.toml"));
        raw_cargo_toml.read_to_string(&mut cargo_toml_contents)?;
    }

    let mut parsed_cargo_toml = cargo_toml_contents
        .parse::<Document>()
        .unwrap_or_else(|_| log_panic("Cargo.toml is not valid TOML"));

    let version = parsed_cargo_toml["package"]["version"]
        .as_str()
        .unwrap_or_else(|| log_panic("'package.version' field not found in Cargo.toml"));

    let mut version =
        semver::Version::parse(version).unwrap_or_else(|_| log_panic("could not parse version"));
    log::info!("Current version: {}", version);

    log::debug!("Getting semver-check version");
    let new_semver_version = get_semver_changes(&version, &args.path)
        .unwrap_or_else(|_| log_panic("could not get semver changes"));
    log::debug!("Possible semver-check version: {}", new_semver_version);

    log::debug!("Getting git version");
    let new_git_version =
        get_git_changes(args.path).unwrap_or_else(|_| log_panic("could not get git releases"));
    log::debug!("Possible git version: {}", new_git_version);

    bump_version(&mut version, &new_semver_version, &new_git_version);
    log::info!("New version: {}", version);

    parsed_cargo_toml["package"]["version"] = toml_edit::value(version.to_string());

    if args.dry_run {
        log::info!("Dry run, not writing new version to Cargo.toml");
    } else {
        log::debug!("Writing new version to Cargo.toml");

        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&cargo_toml_path)
            .unwrap_or_else(|_| log_panic("Could not open Cargo.toml for writing"));

        file.write_all(parsed_cargo_toml.to_string().as_bytes())
            .unwrap_or_else(|_| log_panic("Could not write Cargo.toml"));
    }

    Ok(())
}

fn bump_version(
    current_version: &mut semver::Version,
    semver_version: &semver::Version,
    git_version: &semver::Version,
) {
    let new_version = if semver_version > git_version {
        semver_version
    } else {
        git_version
    };

    current_version.major = new_version.major;
    current_version.minor = new_version.minor;
    current_version.patch = new_version.patch;
}

fn get_git_changes(path: PathBuf) -> Result<semver::Version> {
    let repo = Repository::init(path)?;
    let releases = get_git_releases(&repo)?;

    if let Some(last_version) = releases.last() {
        Ok(semver::Version::parse(
            &last_version.calculate_next_version()?,
        )?)
    } else {
        Ok(semver::Version::new(0, 1, 0))
    }
}

fn get_git_releases(repo: &Repository) -> Result<Vec<Release<'_>>> {
    let mut releases = vec![Release::default()];
    let mut release_index = 0;
    let mut previous_release = Release::default();

    let commits = repo.commits(None, None, None)?;

    for git_commit in commits.into_iter().rev() {
        let commit = Commit::from(&git_commit);
        let commit_id = commit.id.to_string();

        releases[release_index].commits.push(commit);

        let tags = repo.tags(&None, false)?;

        if let Some(tag) = tags.get(&commit_id) {
            releases[release_index].version = Some(tag.to_string());
            releases[release_index].commit_id = Some(commit_id);
            releases[release_index].timestamp = git_commit.time().seconds();
            previous_release.previous = None;
            releases[release_index].previous = Some(Box::new(previous_release));
            previous_release = releases[release_index].clone();
            releases.push(Release::default());
            release_index += 1;
        }
    }

    if release_index > 1 {
        previous_release.previous = None;
        releases[release_index].previous = Some(Box::new(previous_release));
    }

    Ok(releases)
}

fn get_semver_changes(
    current_version: &semver::Version,
    path: &PathBuf,
) -> Result<semver::Version> {
    let current = Rustdoc::from_root(path);
    let check = Check::new(current);

    let result = match check.check_release() {
        Ok(check_result) => {
            let (_, report) = check_result
                .crate_reports()
                .iter()
                .next()
                .unwrap_or_else(|| log_panic("no crate reports"));

            Ok(report.required_bump())
        }
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )),
    }?;

    Ok(bump_version_by_release_type(current_version, &result))
}

fn bump_version_by_release_type(
    version: &semver::Version,
    release_type: &Option<ReleaseType>,
) -> semver::Version {
    match release_type {
        Some(ReleaseType::Major) => semver::Version::new(version.major + 1, 0, 0),
        Some(ReleaseType::Minor) => semver::Version::new(version.major, version.minor + 1, 0),
        _ => semver::Version::new(version.major, version.minor, version.patch + 1),
    }
}
fn log_panic(message: &str) -> ! {
    log::error!("{message}");
    panic!("{message}");
}
