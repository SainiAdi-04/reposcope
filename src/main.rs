mod cli;
mod config;
mod error;
mod model;
mod scanner;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};
use std::path::Path;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Add { path }) => match config::add_project(&path) {
            Ok(()) => println!("added {path}"),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::Remove { name_or_path }) => match config::remove_project(&name_or_path) {
            Ok(()) => println!("removed {name_or_path}"),
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::List) => match config::list_projects() {
            Ok(projects) => {
                if projects.is_empty() {
                    println!("no projects registered");
                } else {
                    for p in &projects {
                        println!("{}\t{}", p.name, p.path.display());
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        None => match config::list_projects()
            .and_then(|projects| tui::run(projects).map_err(|e| error::Error::Tui(e.to_string())))
        {
            Ok(()) => {}
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Some(Commands::DebugScan { path }) => {
            let status = scanner::scan_one(Path::new(&path));
            println!("{status:#?}");
            if status.error.is_some() {
                std::process::exit(1);
            }
        }
        Some(Commands::DebugScanAll) => match config::list_projects() {
            Ok(projects) => {
                let statuses = scanner::concurrency::scan_all(projects);
                println!("{statuses:#?}");
                if statuses.iter().any(|status| status.error.is_some()) {
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
    }
}
