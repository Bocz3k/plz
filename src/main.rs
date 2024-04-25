use std::time::{SystemTime, Duration, UNIX_EPOCH};
use serde_derive::{Serialize, Deserialize};
use clap::{Arg, Command, ArgAction};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::time::Instant;
use std::process::exit;
use anstyle::AnsiColor;
use reqwest::Client;
use std::path::Path;
use anstyle::Style;
use std::io::Write;
use std::fs;
use std::io;
use toml;

const EXECUTABLE_BLACKLIST: [&str; 3] = ["unins000.exe", "UnityCrashHandler64.exe", "UnityCrashHandler32.exe"];

#[derive(Serialize, Deserialize)]
struct Config {
    games_dir: String,
    check_for_updates: bool,
    autoadd_ignore: Vec<String>
}

#[derive(Deserialize)]
struct Release {
    tag_name: String
}


fn read_file<T: for<'de> serde::Deserialize<'de> + serde::Serialize>(filename: &str, default_content: &str) -> T {
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let exe = std::env::current_exe().unwrap().display().to_string();
    let contents = match fs::read_to_string(remove_after_slash(&exe).to_owned() + filename) {
        Ok(x) => x,
        Err(_) => {
            eprintln!("{error}Could find file `{}`, creating new one", filename);
            let data: T = toml::from_str(default_content).unwrap();
            save_file(filename, &data);
            default_content.to_owned()
        }
    };

    match toml::from_str(&contents) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{error}Unable to load data from `{}`. {}", filename, err);
            exit(1);
        }
    }
}


fn save_file<T: serde::Serialize>(filename: &str, data: &T) {
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let contents = toml::to_string(data).unwrap();
    let exe = std::env::current_exe().unwrap().display().to_string();
    match fs::write(remove_after_slash(&exe).to_owned() + filename, contents) {
        Ok(_) => {},
        Err(err) => eprintln!("{error}Failed to save file `{}`. {}", filename, err)
    }
}


async fn fetch_game_info(name: &str) -> Option<(String, Vec<String>)> {
    let yellow = AnsiColor::BrightYellow.on_default();
    let green = AnsiColor::BrightGreen.on_default().bold();
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let success = format!("{green}success:{green:#} ");
    let bold = Style::new().bold();
    let perf = Instant::now();
    let client = reqwest::Client::builder().user_agent("plz").timeout(Duration::from_secs(5)).build().unwrap();

    let url = format!("https://game3rb.com/{}", name);
    let res = match client.get(&url).send().await {
        Ok(res) => res,
        Err(err) => {
            eprintln!("{error}Error sending response: {}", err);
            return None;
        },
    };

    if res.status().as_u16() == 404 {
        eprintln!("{error}Game not found");
        return None;
    }

    let soup = Html::parse_document(&res.text().await.unwrap());
    let title = soup
        .select(&Selector::parse("h1.post-title").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .replace("Download ", "")
        .replace(" + OnLine", "");

    let item = soup.select(&Selector::parse("a#download-link.direct").unwrap()).next()?;
    let item_href = item.value().attr("href")?;
    let res = match client.get(item_href).send().await {
        Ok(res) => res,
        Err(_) => return None,
    };

    let selected = &Selector::parse("ol li").unwrap();
    let soup = Html::parse_document(&res.text().await.unwrap());
    let links = soup.select(selected);
    let mut items = Vec::new();
    for link in links {
        if let Some(host) = link.select(&Selector::parse("a").unwrap()).next().and_then(|a| a.value().attr("href")) {
            let mut idx = host.find("://").unwrap() + 3;
            if host[idx..].starts_with("www.") {
                idx += 4;
            }
            let dot = host[idx..].find('.').unwrap() + idx;
            let name = titlecase(&host[idx..dot]);
            items.push(format!(" {bold}{name}:{bold:#} {host}"));
        }
    }

    println!("{success}Fetched Game3rb for `{yellow}{}{yellow:#}` in {yellow}{:.2}{yellow:#}s\n", name, perf.elapsed().as_secs_f64());
    Some((title, items))
}


fn titlecase(string: &str) -> String {
    let mut char_list: Vec<char> = string.chars().collect();
    for char in char_list.iter_mut() {
        if char.is_alphabetic() {
            *char = char.to_ascii_uppercase();
            break;
        }
    }
    char_list.into_iter().collect()
}


fn recursive_search(path: &str, folder_path: &str, aliases: &mut HashMap<String, String>, config: &mut Config) -> io::Result<()> {
    let yellow = AnsiColor::BrightYellow.on_default();
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
            if !config.autoadd_ignore.contains(&file_path) && !aliases.values().any(|val| val == &file_path) {
                print!("Alias name for `{yellow}{}{yellow:#}` (enter to skip): ", filename);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let name = input.trim();
                
                if !name.is_empty() {
                    if aliases.contains_key(name) {
                        if user_input(format!("Overwrite alias `{yellow}{}{yellow:#}`? (y/n) ", name)) {
                            aliases.insert(name.to_string(), file_path);
                        }
                    } else {
                        aliases.insert(name.to_string(), file_path);
                    }
                } else {
                    config.autoadd_ignore.push(file_path);
                }
            }
        }
    } else if !folders.is_empty() {
        println!("No executables found in `{yellow}{}{yellow:#}`, going into subfolders", path);
        for folder in folders {
            let folder_path = folder.display().to_string();
            recursive_search(&folder_path, &folder_path, aliases, config)?;
        }
    }

    Ok(())
}


fn autoadd(aliases: &mut HashMap<String, String>, config: &mut Config) -> io::Result<()> {
    let yellow = AnsiColor::BrightYellow.on_default();
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
        } else if file_path.is_file() && file_name.ends_with(".exe") &&
                  !config.autoadd_ignore.contains(&file_path.display().to_string()) &&
                  !aliases.values().any(|val| val == &file_path.display().to_string()) {
                
                print!("Alias name for `{yellow}{}{yellow:#}` (enter to skip): ", file_name);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let name = input.trim();
                
                if !name.is_empty() {
                    if aliases.contains_key(name) {
                        if user_input(format!("Overwrite alias `{yellow}{}{yellow:#}`? (y/n) ", name)) {
                            aliases.insert(name.to_string(), file_path.display().to_string());
                        }
                    } else {
                        aliases.insert(name.to_string(), file_path.display().to_string());
                    }
                } else {
                    config.autoadd_ignore.push(file_path.display().to_string());
                }
        }
    }

    save_file("config.toml", &config);
    save_file("aliases.toml", &aliases);
    Ok(())
}


fn check_config(config: &mut Config, aliases: &mut HashMap<String, String>) {
    let yellowb = AnsiColor::BrightYellow.on_default().bold();
    let yellow = AnsiColor::BrightYellow.on_default();
    let warning = format!("\n{yellowb}warning:{yellowb:#} ");
    let path = Path::new(&config.games_dir);
    
    if !path.exists() {
        eprint!("{warning}games_dir `{yellow}{}{yellow:#}` does not exist.", config.games_dir);
    } else if !path.is_dir() {
        eprint!("{warning}games_dir `{yellow}{}{yellow:#}` is not a directory.", config.games_dir);
    } else if !config.games_dir.contains(std::path::MAIN_SEPARATOR) {
        eprint!("{warning}games_dir `{yellow}{}{yellow:#}` doesn't use system's main separator ({}).", config.games_dir, std::path::MAIN_SEPARATOR);
    }

    for path in aliases {
        if !Path::new(path.1).exists() {
            eprint!("{warning}Alias `{yellow}{}{yellow:#}` points to `{yellow}{}{yellow:#}` which does not exist.", path.0, path.1);
        } else if !Path::new(path.1).is_file() {
            eprint!("{warning}Alias `{yellow}{}{yellow:#}` points to `{yellow}{}{yellow:#}` which is not a file.", path.0, path.1);
        }
    }
}


fn remove_after_slash(string: &str) -> &str {
    let mut character = &'/';
    if string.find('\\') != None {
        character = &'\\';
    }
    let char_list: Vec<char> = string.chars().collect();
    for (i, char) in char_list.iter().enumerate().rev() {
        if char == character {
            return &string[..i + 1];
        }
    }
    string
}


fn sort_by_key_length(mut hashmap: HashMap<String, String>) -> Vec<(String, String)> {
    let mut vec: Vec<(String, String)> = hashmap.drain().collect();
    vec.sort_by(|(key1, _), (key2, _)| key2.len().cmp(&key1.len()));
    vec.into_iter().collect()
}


fn user_input(message: String) -> bool {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    }

    let input = buf.trim().to_lowercase();
    if input == "y" || input == "yes" {
        return true;
    }
    return false;
}


async fn check_for_updates() -> String {
    let client = Client::builder().user_agent("plz").timeout(Duration::from_secs(5)).build().unwrap();
    let res = client.get("https://api.github.com/repos/Bocz3k/plz/releases/latest").send().await;
    if let Ok(res) = res {
        let release: Release = res.json().await.unwrap();
        if release.tag_name != String::from("v") + env!("CARGO_PKG_VERSION") {
            let green = AnsiColor::BrightGreen.on_default().bold();
            let yellow = AnsiColor::BrightYellow.on_default().bold();
            return format!("\n{green}New version of plz available:{green:#}\n Current: {yellow}v{}{yellow:#}\n New version: {green}{}{green:#}\n", env!("CARGO_PKG_VERSION"), release.tag_name);
        }
    }
    String::new()
}


#[tokio::main]
async fn main() {
    let mut config: Config = read_file("config.toml", "games_dir = \"\"\ncheck_for_updates = true\nautoadd_ignore = []\n");
    let mut aliases: HashMap<String, String> = read_file("aliases.toml", "");

    let update_message = match config.check_for_updates {
        true => check_for_updates().await,
        false => String::new(),
    };

    let matches = Command::new("plz")
        .about("plz is an alias manager to help you manage your games.")
        .version("0.2.2")
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
            Command::new("config")
                .about("Manage your config settings or view them")
                .subcommand(
                    Command::new("check_for_updates")
                        .about("Change or view check_for_updates in your config")
                        .arg(
                            Arg::new("value")
                                .help("Value to change it to")
                        )
                )
                .subcommand(
                    Command::new("games_dir")
                        .about("Change or view games_dir in your config")
                        .arg(
                            Arg::new("value")
                                .help("Value to change it to")
                        )
                )
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
                        )
                        .arg(
                            Arg::new("path")
                                .help("The path to the alias")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("remove")
                        .about("Remove an alias")
                        .arg(
                            Arg::new("alias")
                                .help("The alias to remove")
                                .required(true)
                        )
                )
                .subcommand(
                    Command::new("list")
                        .about("List all aliases")
                )
                .subcommand(
                    Command::new("autoadd")
                        .about("Automatically add aliases from games_dir")
                )
        )
        .subcommand(
            Command::new("fetch")
                .about("Fetch links from fetch sites and info site")
                .arg(
                    Arg::new("game")
                        .help("The game to fetch links for")
                        .required(true)
                )
        )
    .try_get_matches();

    let mut execute = true;
    let mut error: Option<clap::Error> = None;
    let mut command: Option<clap::ArgMatches> = None;
    match matches {
        Ok(matches) => command = Some(matches),
        Err(err) => {
            execute = false;
            error = Some(err);
        }
    };

    if execute {
        let red = AnsiColor::BrightRed.on_default().bold();
        let error = format!("{red}error:{red:#} ");
        let green = AnsiColor::BrightGreen.on_default().bold();
        let success = format!("{green}success:{green:#} ");
        let yellow = AnsiColor::BrightYellow.on_default();
        let bold = Style::new().bold();
        match command.unwrap().subcommand() {
            Some(("run", matches)) => {
                let alias: &String = matches.get_one("alias").unwrap();
                match aliases.get(alias) {
                    Some(path) => {
                        match std::env::set_current_dir(remove_after_slash(path)) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("{error}Path: {} | {}", remove_after_slash(path), err);
                                exit(1);
                            }
                        }
                        if let Err(err) = std::process::Command::new(path).status() {
                            eprintln!("{error}Failed to run alias `{yellow}{}{yellow:#}`: {}", alias, err);
                        }
                    },
                    None => eprintln!("{error}Alias `{yellow}{}{yellow:#}` not found", alias)
                }
            }
            Some(("random", _)) => {
                if aliases.len() == 0 {
                    eprintln!("{error}No aliases found");
                    exit(1);
                }
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let index = current_time % aliases.len() as u64;
                if let Some((key, value)) = aliases.iter().nth(index as usize) {
                    match std::env::set_current_dir(remove_after_slash(value)) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{error}Path: {} | {}", remove_after_slash(value), err);
                            exit(1);
                        }
                    }
                    if let Err(err) = std::process::Command::new(value).status() {
                        eprintln!("{error}Failed to run alias `{yellow}{}{yellow:#}`: {}", key, err);
                    } 
                }
            }
            Some(("config", matches)) => {
                match matches.subcommand() {
                    Some(("check_for_updates", matches)) => {
                        let value: Option<&String> = matches.get_one("value");
                        if value.is_some() {
                            if value.unwrap() == "true" {
                                config.check_for_updates = true;
                                save_file("config.toml", &config);
                                println!("{success}Set value of check_for_updates to `{yellow}true{yellow:#}`");
                            } else if value.unwrap() == "false" {
                                config.check_for_updates = false;
                                save_file("config.toml", &config);
                                println!("{success}Set value of check_for_updates to `{yellow}false{yellow:#}`");
                            } else {
                                eprintln!("{error}Value needs to be either `{yellow}false{yellow:#}` or `{yellow}true{yellow:#}`");
                            }
                        } else {
                            println!("Current value of check_for_updates is `{yellow}{}{yellow:#}`", config.check_for_updates);
                        }
                    }
                    Some(("games_dir", matches)) => {
                        let value: Option<&String> = matches.get_one("value");
                        if value.is_some() {
                            config.games_dir = value.unwrap().clone();
                            save_file("config.toml", &config);
                            println!("{success}Set value of games_dir to `{yellow}{}{yellow:#}`", value.unwrap());
                        } else {
                            println!("Current value of games_dir is `{yellow}{}{yellow:#}`", config.games_dir);
                        }
                    }
                    _ => {
                        println!("{bold}Current config values:{bold:#}");
                        println!("{bold}games_dir:{bold:#} `{yellow}{}{yellow:#}`", config.games_dir);
                        println!("{bold}check_for_updates:{bold:#} `{yellow}{}{yellow:#}`", config.check_for_updates);
                    }
                }
            }
            Some(("alias", matches)) => {
                match matches.subcommand() {
                    Some(("add", matches)) => {
                        let alias: &String = matches.get_one("alias").unwrap();
                        let path: &String = matches.get_one("path").unwrap();

                        if aliases.contains_key(alias) {
                            if user_input(format!("Overwrite alias `{yellow}{}{yellow:#}`? (y/n) ", alias)) {
                                aliases.insert(alias.to_string(), path.to_string());
                                save_file("aliases.toml", &aliases);
                                println!("{success}Overwrote alias `{yellow}{}{yellow:#}`", alias);
                            }
                        } else {
                            aliases.insert(alias.to_string(), path.to_string());
                            save_file("aliases.toml", &aliases);
                            println!("{success}Added alias `{yellow}{}{yellow:#}`", alias);
                        }

                    }
                    Some(("remove", matches)) => {
                        let alias: &String = matches.get_one("alias").unwrap();
                        if aliases.contains_key(alias) {
                            aliases.remove(alias);
                            save_file("aliases.toml", &aliases);
                            println!("{success}Removed alias `{yellow}{}{yellow:#}`", alias);
                        } else {
                            println!("{error}Alias `{yellow}{}{yellow:#}` doesn't exist", alias);
                        }
                    }
                    Some(("list", _)) => {
                        let sorted = sort_by_key_length(aliases.clone());
                        
                        let gray = AnsiColor::BrightBlack.on_default();

                        println!("{bold}Aliases:");
                        for (alias, path) in sorted.iter() {
                            println!(" {bold}{}{bold:#} {gray}->{gray:#} {}", alias, path);
                        }
                    }
                    Some(("autoadd", _)) => {
                        match autoadd(&mut aliases, &mut config) {
                            Ok(_) => {
                                save_file("config.toml", &config);
                                save_file("aliases.toml", &aliases);
                            },
                            Err(err) => eprintln!("{error}{}", err)
                        }
                    }
                    _ => unreachable!(),
                }
            }
            Some(("fetch", matches)) => {
                let game: &String = matches.get_one("game").unwrap();
                if let Some((title, items)) = fetch_game_info(game).await {
                    println!("{bold}{}", title);
                    println!("Download Links:{bold:#}");
                    for item in items {
                        println!("{item}");
                    }
                }
            }
            _ => unreachable!(),
        }
    }
    if !execute {
        let err = error.unwrap();
        let _ = err.print();
    }
    check_config(&mut config, &mut aliases);
    print!("{}", update_message);
}
