use std::fmt;
use std::process::Command;

use anyhow::{bail, Result};
// use semver::Version;

pub enum PartType {
    Major,
    Minor,
    Patch,
}

pub struct Version {
    pub version: semver::Version,
    pub v_prefix: bool,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.v_prefix {
            write!(f, "v{}", self.version)
        } else {
            write!(f, "{}", self.version)
        }
    }
}

impl Version {
    pub fn parse(version: &str) -> Result<Version> {
        let ver = semver::Version::parse(strip_v(version))?;
        Ok(Version {
            version: ver,
            v_prefix: version.starts_with('v'),
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
    let tags: Result<Vec<Version>> = all_tags
        .split('\n')
        .filter(|tag| !tag.is_empty()) // Filter out any empty tags.
        .map(Version::parse)
        .collect(); // Collect results into a Vec<Version>.
    tags
}

pub fn bump(b: &Bump) -> Result<()> {
    let mut version = get_latest_tag()?;

    // Bump the given version and set the lower parts to 0.
    match b.version_type {
        PartType::Major => {
            version.version.major += b.number;
            version.version.minor = 0;
            version.version.patch = 0;
        }
        PartType::Minor => {
            version.version.minor += b.number;
            version.version.patch = 0;
        }
        PartType::Patch => {
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

pub fn push_latest() -> Result<String> {
    let latest_tag = get_latest_tag()?;
    let result = run(&format!("git push origin {}", latest_tag))?;
    Ok(result)
}

pub fn delete_latest() -> Result<String> {
    let latest_tag = get_latest_tag()?;
    let result = run(&format!("git tag -d {}", latest_tag))?;
    Ok(result)
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

/// run executes the given command and returns the output from the command.
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
    tag.strip_prefix('v').unwrap_or(tag)
}
