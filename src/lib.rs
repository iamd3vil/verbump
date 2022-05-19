use std::fmt;
use std::process::Command;

use anyhow::{bail, Result};
// use semver::Version;

pub enum PartType {
    MAJOR,
    MINOR,
    PATCH,
}

pub struct Version {
    pub version: semver::Version,
    pub v_prefix: bool,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fmt_string = format!("{}", self.version);
        if self.v_prefix {
            fmt_string = format!("v{}", self.version);
        }
        write!(f, "{}", fmt_string)
    }
}

impl Version {
    pub fn parse(version: &str) -> Result<Version> {
        let ver = semver::Version::parse(strip_v(&version))?;
        Ok(Version {
            version: ver,
            v_prefix: version.starts_with("v"),
        })
    }
}

pub struct Bump<'a> {
    pub version_type: PartType,
    pub number: u64,
    pub suffix: &'a str,
}

pub fn get_latest_tag() -> Result<Version> {
    let latest_tag = run("git describe --abbrev=0 --tags")?;
    let version = Version::parse(&latest_tag)?;
    Ok(version)
}

pub fn get_all_tags() -> Result<Vec<Version>> {
    let all_tags = run("git tag --sort=-refname")?;
    let all_tags: Vec<&str> = all_tags.split('\n').collect();
    let mut tags: Vec<Version> = Vec::with_capacity(all_tags.len());
    for tag in all_tags {
        let version = Version::parse(tag)?;
        tags.push(version);
    }
    Ok(tags)
}

pub fn bump(b: &Bump) -> Result<()> {
    let mut version = get_latest_tag()?;

    // Bump the given version and set the lower parts to 0.
    match b.version_type {
        PartType::MAJOR => {
            version.version.major += b.number;
            version.version.minor = 0;
            version.version.patch = 0;
        }
        PartType::MINOR => {
            version.version.minor += b.number;
            version.version.patch = 0;
        }
        PartType::PATCH => {
            version.version.patch += b.number;
        }
    }

    // Set latest tag.
    if !b.suffix.is_empty() {
        run(&format!(
            "git tag -a v{}-{} -m v{}-{}",
            version.version, b.suffix, version.version, b.suffix
        ))?;
    } else {
        run(&format!(
            "git tag -a v{} -m v{}",
            version.version, version.version
        ))?;
    }

    Ok(())
}

pub fn init() -> Result<()> {
    // Check if there's already a tag.
    if let Ok(tag) = get_latest_tag() {
        println!("tag already initialized: {}", tag);
        return Ok(());
    }
    run("git tag -a v0.1.0 -m v0.1.0")?;
    println!("tag initialized with v0.1.0");
    Ok(())
}

fn run(cmd: &str) -> Result<String> {
    let args: Vec<&str> = cmd.split(' ').collect();

    let mut command = Command::new(args[0]);
    if args.len() > 1 {
        command.args(&args[1..]);
    }

    let output = command.output()?;
    if !output.status.success() {
        bail!(
            "error running command: `{}`: {}",
            cmd,
            String::from_utf8_lossy(&output.stderr).to_string()
        );
    }

    Ok(String::from(
        String::from_utf8_lossy(&output.stdout).to_string().trim(),
    ))
}

fn strip_v(tag: &str) -> &str {
    match tag.strip_prefix('v') {
        Some(t) => t,
        None => tag,
    }
}
