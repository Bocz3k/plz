use titlecase::titlecase;
use clap::{Arg, Command, ArgAction};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Instant;
use rand::seq::SliceRandom;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::process::exit;
use toml;
use std::fs;
use std::io;

const EXECUTABLE_BLACKLIST: [&str; 3] = ["unins000.exe", "UnityCrashHandler64.exe", "UnityCrashHandler32.exe"];

#[derive(Deserialize)]
struct Config {
    games_dir: String,
    autoadd_ignore: Vec<String>
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


fn fetch_game_info(name: &str) -> Option<(String, String, String, Vec<String>)> {
    let perf = Instant::now();
    let client = Client::new();

    let url = format!("https://game3rb.com/{}", name);
    let res = match client.get(&url).send() {
        Ok(res) => res,
        Err(_) => return None,
    };

    if res.status().is_client_error() {
        return None;
    }

    let soup = Html::parse_document(&res.text().unwrap());
    let game_name = soup
        .select(&Selector::parse("h1.post-title").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .trim_start_matches("Game:")
        .trim()
        .to_lowercase();

    let version;
    let mut game_name = game_name.clone();
    if game_name.contains(name) {
        version = game_name.replace(name, "").replace("+ online", "").trim().to_string();
        game_name = name.to_string();
    } else {
        println!("Game name not found");
        return None;
    }

    let size = soup
        .select(&Selector::parse("strong").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .replace("RELEASE", "")
        .replace("SIZE", "Size:")
        .replace("::", ":")
        .trim()
        .to_string();

    let item = soup.select(&Selector::parse("a#download-link").unwrap()).next()?;
    let item_href = item.value().attr("href")?;
    let res = match client.get(item_href).send() {
        Ok(res) => res,
        Err(_) => return None,
    };

    let selected = &Selector::parse("ol li").unwrap();
    let soup = Html::parse_document(&res.text().unwrap());
    let links = soup.select(selected);
    let mut items = Vec::new();
    for link in links {
        // TODO: Find a better way to do this, hosts are not correctly named
        if let Some(host) = link.select(&Selector::parse("a").unwrap()).next().and_then(|a| a.value().attr("href")) {
            let idx = host.find("://").map(|i| i + 3).unwrap_or(0);
            let idx = idx + if host[idx..].starts_with("www.") { 4 } else { 0 };
            let slash = host[idx..].find('/').unwrap_or_else(|| host.len());
            let name = titlecase(host[idx..slash].trim_end_matches('.'));
            items.push(format!("{}: {}", name, host));
        }
    }

    println!("Fetched Game3rb for {} in {:.2}s", name, perf.elapsed().as_secs_f64());
    Some((version, size, game_name, items))
}


fn recursive_search(path: &str, folder_path: &str, aliases: &mut HashMap<String, String>, config: &mut Config) -> io::Result<()> {
    let mut executables = Vec::new();
    let mut folders = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = entry.file_name().into_string().unwrap_or_default();

        if file_path.is_file()
            && file_name.ends_with(".exe")
            && !EXECUTABLE_BLACKLIST.contains(&file_name.as_str())
        {
            executables.push(file_path);
        } else if file_path.is_dir() {
            folders.push(file_path);
        }
    }

    if !executables.is_empty() {
        for executable_file in executables {
            let file_path = executable_file.display().to_string();
            let filename = executable_file.file_name().unwrap().to_string_lossy();
            println!("Alias name for {} (enter to skip): ", filename);
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let name = input.trim();
            if !name.is_empty() {
                aliases.insert(name.to_string(), file_path);
            } else {
                config.autoadd_ignore.push(file_path);
            }
        }
    } else if !folders.is_empty() {
        println!("No executables found in {}, going into subfolders", path);
        for folder in folders {
            let folder_path = folder.display().to_string();
            recursive_search(&folder_path, &folder_path, aliases, config)?;
        }
    }

    Ok(())
}


fn autoadd(aliases: &mut HashMap<String, String>, config: &mut Config) -> io::Result<()> {
    if config.games_dir.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "games_dir is empty, please set it first.",
        ));
    }

    for entry in fs::read_dir(&config.games_dir)? {
        let entry = entry?;
        let file_path = entry.path();
        let file_name = entry.file_name().into_string().unwrap_or_default();

        if file_path.is_dir() {
            let folder_path = file_path.display().to_string();
            recursive_search(&file_name, &folder_path, aliases, config)?;
        } else if file_path.is_file() && file_name.ends_with(".exe") {
            println!("Alias name for {} (enter to skip): ", file_name);
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let name = input.trim();
            if !name.is_empty() {
                aliases.insert(name.to_string(), file_path.display().to_string());
            }
        }
    }

    Ok(())
}


fn main() {
    // TODO: Add a function to check if the config file is corrupted, and fix if it is.
    let mut config: Config = read_file("config.toml", "games_dir = \"\"\nautoadd_ignore = []\n");
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
                .subcommand(
                    Command::new("remove")
                        .about("Remove an alias")
                        .arg(
                            Arg::new("alias")
                                .help("The alias to remove")
                                .required(true)
                                .action(ArgAction::Set)
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List all aliases")
                )
                .subcommand(
                    Command::new("autoadd")
                        .about("Automatically add aliases from games directory")
                )
        )
        .subcommand(
            Command::new("fetch")
                .about("Fetch links from fetch sites and info site")
                .arg(
                    Arg::new("game")
                        .help("The game to fetch links for")
                        .required(true)
                        .action(ArgAction::Set)
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
                    aliases.insert(alias.to_string(), path.to_string());
                }
                Some(("remove", matches)) => {
                    let alias: &String = matches.get_one("alias").unwrap();
                    aliases.remove(alias);
                }
                Some(("list", _)) => {
                    println!("Aliases:");
                    for (alias, path) in aliases.iter() {
                        println!("{} -> {}", alias, path);
                    }
                }
                Some(("autoadd", _)) => {
                    match autoadd(&mut aliases, &mut config) {
                        Ok(_) => (),
                        Err(err) => eprintln!("{}", err)
                    }
                }
                _ => unreachable!(),
            }
        }
        Some(("fetch", matches)) => {
            let game: &String = matches.get_one("game").unwrap();
            if let Some((version, size, game_name, items)) = fetch_game_info(game) {
                println!("Game Name: {}", game_name);
                println!("Version: {}", version);
                println!("Size: {}", size);
                println!("Download Links:");
                for item in items {
                    println!("{}", item);
                }
            } else {
                println!("Game not found.");
            }
        }
        _ => unreachable!(),
    }
}
