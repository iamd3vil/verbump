use std::process::Command;

use anyhow::{bail, Context, Result};
use semver::Version;

pub struct Bump<'a> {
    pub minor: u64,
    pub major: u64,
    pub patch: u64,
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
    let latest_tag = get_latest_tag().with_context(|| format!("error getting latest tag"))?;

    let version = Version::new(
        latest_tag.major + b.major,
        latest_tag.minor + b.minor,
        latest_tag.patch + b.patch,
    );

    // Set latest tag.
    run(&format!("git tag -a v{} -m v{}", version, version))?;

    Ok(())
}

pub fn init() -> Result<()> {
    run("git tag -a v0.1.0 -m v0.1.0")?;
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
