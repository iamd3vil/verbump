use anyhow::Result;
use gumdrop::Options;
use std::{env, process};
use verbump::{get_latest_tag, init};

#[derive(Options, Debug)]
struct Args {
    #[options(help = "prints help message")]
    help: bool,

    #[options(help = "increments minor version")]
    minor: bool,

    #[options(help = "increments major version")]
    major: bool,

    #[options(help = "increments patch version")]
    patch: bool,

    #[options(help = "shows the latest tag")]
    show: bool,

    #[options(help = "inits a tag")]
    init: bool,
}

fn main() {
    let os_args: Vec<String> = env::args().collect();
    if os_args.len() == 1 {
        print_help_and_exit(&os_args);
    }

    let args = Args::parse_args_default_or_exit();

    match args {
        Args { show: true, .. } => {
            let latest_tag = handle_error(get_latest_tag());
            println!("{}", latest_tag);
        }
        Args { init: true, .. } => {
            handle_error(init());
        }
        _ => {
            print_help_and_exit(&os_args);
            println!("invalid flag")
        }
    }
}

fn print_help_and_exit(os_args: &Vec<String>) {
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
