use anyhow::Result;
use gumdrop::Options;
use std::{env, process};
use verbump::{bump, delete_latest, get_all_tags, get_latest_tag, init, push_latest, Bump};

#[derive(Options, Debug)]
struct Args {
    #[options(help = "prints help message")]
    help: bool,

    #[options(help = "increments minor version")]
    minor: bool,

    #[options(help = "increments major version")]
    major: bool,

    #[options(help = "increments patch version", default = "true")]
    patch: bool,

    #[options(help = "shows the latest tag")]
    show: bool,

    #[options(help = "inits a tag")]
    init: bool,

    #[options(help = "shows all tags")]
    all: bool,

    #[options(help = "pushes the latest tag")]
    push_latest: bool,

    #[options(help = "deletes latest tag")]
    delete_latest: bool,
}

fn main() {
    let os_args: Vec<String> = env::args().collect();
    let args = Args::parse_args_default_or_exit();

    match args {
        Args { show: true, .. } => {
            let latest_tag = handle_error(get_latest_tag());
            println!("{}", latest_tag);
        }
        Args {
            delete_latest: true,
            ..
        } => {
            let latest_tag = handle_error(get_latest_tag());
            handle_error(delete_latest());
            println!("deleted {}", latest_tag);
        }
        Args {
            push_latest: true, ..
        } => {
            handle_error(push_latest());
            println!("pushed latest tag");
        }
        Args { init: true, .. } => {
            handle_error(init());
        }
        Args { all: true, .. } => {
            let all_tags = handle_error(get_all_tags());
            for tag in all_tags {
                println!("{}", tag);
            }
        }
        Args { major: true, .. } => {
            handle_error(bump(&Bump {
                version_type: verbump::PartType::Major,
                number: 1,
                suffix: "",
            }));
            let latest_tag = handle_error(get_latest_tag());
            println!("bumped to {}", latest_tag);
        }
        Args { minor: true, .. } => {
            handle_error(bump(&Bump {
                version_type: verbump::PartType::Minor,
                number: 1,
                suffix: "",
            }));
            let latest_tag = handle_error(get_latest_tag());
            println!("bumped to {}", latest_tag);
        }
        Args { patch: true, .. } => {
            handle_error(bump(&Bump {
                version_type: verbump::PartType::Patch,
                number: 1,
                suffix: "",
            }));
            let latest_tag = handle_error(get_latest_tag());
            println!("bumped to {}", latest_tag);
        }
        _ => {
            print_help_and_exit(&os_args);
            println!("invalid flag")
        }
    }
}

fn print_help_and_exit(os_args: &[String]) {
    eprintln!("Usage: {} [OPTIONS]", os_args[0]);
    eprintln!();
    eprintln!("{}", Args::usage());
    process::exit(1);
}

fn handle_error<T>(res: Result<T>) -> T {
    match res {
        Ok(t) => t,
        Err(err) => {
            println!("{}", err);
            process::exit(1);
        }
    }
}
