use clap::{Arg, Command, ArgAction};
use rand::seq::SliceRandom;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::process::exit;
use toml;

#[derive(Deserialize)]
struct Config {
    games_dir: String,
    fetch_sites: Vec<String>,
    info_site: String
}

fn read_file<T: for<'de> serde::Deserialize<'de>>(filename: &str, default_content: &str) -> T {
    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`, creating new one", filename.replace(".toml", ""));
            let _ = fs::write(filename, default_content);
            default_content.to_string()
        }
    };

    match toml::from_str(&contents) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Unable to load data from `{}`", filename);
            exit(1);
        }
    }
}


fn main() {
    let config: Config = read_file("config.toml", "games_dir = \"\"\nfetch_sites = []\ninfo_site = \"\"");
    let mut aliases: HashMap<String, String> = read_file("aliases.toml", "");

    let matches = Command::new("plz")
        .about("plz is an alias manager to help you manage your games.")
        .version("0.1.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run an alias")
                .arg(
                    Arg::new("alias")
                        .help("The alias to run")
                        .required(true)
                        .action(ArgAction::Set)
                )
        )
        .subcommand(
            Command::new("random")
                .about("Run a random alias")
        )
        .subcommand(
            Command::new("alias")
                .about("Manage aliases")
                .subcommand_required(true)
                .subcommand(
                    Command::new("add")
                        .about("Add a new alias")
                        .arg(
                            Arg::new("alias")
                                .help("The alias to add")
                                .required(true)
                                .action(ArgAction::Set)
                        )
                        .arg(
                            Arg::new("path")
                                .help("The path to the alias")
                                .required(true)
                                .action(ArgAction::Set)
                        )
                )
        )
        .get_matches();
    
    match matches.subcommand() {
        Some(("run", matches)) => {
            let alias: &String = matches.get_one("alias").unwrap();
            match aliases.get(alias) {
                Some(path) => {
                    if let Err(err) = std::process::Command::new(path).status() {
                        eprintln!("Failed to run alias `{}`: {}", alias, err);
                    }
                }
                None => eprintln!("Alias `{}` not found", alias)
            }
        }
        Some(("random", _)) => {
            let paths_vec: Vec<(&String, &String)> = aliases.iter().collect();
            if let Some((alias, path)) = paths_vec.choose(&mut rand::thread_rng()) {
                if let Err(err) = std::process::Command::new(path).status() {
                    eprintln!("Failed to run alias {}: {}", alias, err);
                }
            } else {
                println!("No aliases found.");
            }
        }
        Some(("alias", matches)) => {
            match matches.subcommand() {
                Some(("add", matches)) => {
                    let alias: &String = matches.get_one("alias").unwrap();
                    let path: &String = matches.get_one("path").unwrap();
                    let alias = alias.to_owned();
                    let path = path.to_owned();
                    aliases.insert(alias, path);
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}
