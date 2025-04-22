use std::fmt;
use std::io::Write; // Added for run_with_stdin
use std::process::{Command, Stdio}; // Added Stdio for run_with_stdin

use anyhow::{bail, Result};
// use semver::Version;

pub enum PartType {
    Major,
    Minor,
    Patch,
}

// Add Clone derive
#[derive(Clone)]
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
    let latest_tag = run("git", &["describe", "--abbrev=0", "--tags"])?;
    let version = Version::parse(&latest_tag)?;
    Ok(version)
}

pub fn get_all_tags() -> Result<Vec<Version>> {
    // Use the refactored run function
    let all_tags_output = run("git", &["tag", "--sort=-refname"])?;
    let tags: Result<Vec<Version>> = all_tags_output
        .split('\n')
        .filter(|tag| !tag.is_empty()) // Filter out any empty tags.
        .map(Version::parse)
        .collect(); // Collect results into a Vec<Version>.
    tags
}

pub fn bump(b: &Bump) -> Result<()> {
    // Get the previous version *before* calculating the new one
    let previous_version = get_latest_tag()?;
    let mut new_version = previous_version.clone(); // Clone to modify

    // Bump the *cloned* version and set the lower parts to 0.
    match b.version_type {
        PartType::Major => {
            new_version.version.major += b.number;
            new_version.version.minor = 0;
            new_version.version.patch = 0;
        }
        PartType::Minor => {
            new_version.version.minor += b.number;
            new_version.version.patch = 0;
        }
        PartType::Patch => {
            new_version.version.patch += b.number;
        }
    }

    // Format the new tag string (always use 'v' prefix for the tag name)
    let new_tag_str = if !b.suffix.is_empty() {
        format!("v{}-{}", new_version.version, b.suffix)
    } else {
        format!("v{}", new_version.version)
    };

    // Generate the commit log message
    // Use previous_version.to_string() to get the tag name (e.g., "v0.1.0")
    let commit_log = get_commit_log_between(&previous_version.to_string(), "HEAD")?;

    // Construct the final tag message
    let tag_message = format!(
        "Latest release: {}\n\n{}",
        new_tag_str, // Use the calculated new tag string here
        commit_log
    );

    // Create the annotated tag using the generated message via stdin
    // Use run_with_stdin and -F - to pass the message safely
    run_with_stdin(
        "git",
        &["tag", "-a", &new_tag_str, "-F", "-"], // Use -F - to read from stdin
        &tag_message,
    )?;

    Ok(())
}

pub fn push_latest() -> Result<String> {
    let latest_tag = get_latest_tag()?;
    // Use the refactored run function
    let result = run("git", &["push", "origin", &latest_tag.to_string()])?;
    Ok(result)
}

pub fn delete_latest() -> Result<String> {
    let latest_tag = get_latest_tag()?;
    // Use the refactored run function
    let result = run("git", &["tag", "-d", &latest_tag.to_string()])?;
    Ok(result)
}

pub fn init() -> Result<()> {
    // Check if there's already a tag.
    if let Ok(tag) = get_latest_tag() {
        println!("tag already initialized: {}", tag);
        return Ok(());
    }
    // Use the refactored run function
    run("git", &["tag", "-a", "v0.1.0", "-m", "v0.1.0"])?;
    println!("tag initialized with v0.1.0");
    Ok(())
}

/// Gets the commit log between two references using `git log --oneline --no-merges`.
fn get_commit_log_between(from_ref: &str, to_ref: &str) -> Result<String> {
    let range = format!("{}..{}", from_ref, to_ref);
    // Use --no-merges like the fish script example
    run("git", &["log", &range, "--oneline", "--no-merges"])
}

/// run executes the given command and returns the trimmed stdout output.
fn run(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()?;

    if !output.status.success() {
        bail!(
            "error running command: `{} {}`: {}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).to_string()
        );
    }

    Ok(String::from(String::from_utf8_lossy(&output.stdout).trim()))
}

/// run_with_stdin executes the given command, provides input via stdin, and checks for success.
fn run_with_stdin(program: &str, args: &[&str], stdin_data: &str) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped()) // Pipe stdin
        .stdout(Stdio::piped()) // Capture stdout (optional, but good practice)
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()?;

    // Write the message to the command's stdin
    // The child.stdin is an Option<ChildStdin>, so we need to handle None
    if let Some(mut stdin) = child.stdin.take() {
        // Use write_all to ensure the entire message is written
        stdin.write_all(stdin_data.as_bytes())?;
        // stdin is closed automatically when `stdin` goes out of scope here
    } else {
        bail!(
            "Failed to open stdin for command: `{} {}`",
            program,
            args.join(" ")
        );
    }

    // Wait for the command to finish and get the output
    let output = child.wait_with_output()?;

    if !output.status.success() {
        bail!(
            "error running command with stdin: `{} {}`: {}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).to_string()
        );
    }
    Ok(())
}

fn strip_v(tag: &str) -> &str {
    tag.strip_prefix('v').unwrap_or(tag)
}
