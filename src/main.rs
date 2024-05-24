use std::time::{UNIX_EPOCH, SystemTime, Duration};
use std::path::{MAIN_SEPARATOR, Path};
use serde::{Serialize, Deserialize};
use scraper::{Html, Selector};
use std::collections::HashMap;
use clap::{Arg, Command};
use std::time::Instant;
use std::process::exit;
use anstyle::AnsiColor;
use reqwest::Client;
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
    autoadd_ignore: Vec<String>,
    aliases: HashMap<String, String>,
    // meta: HashMap<String, String>
}

#[derive(Deserialize)]
struct Release {
    tag_name: String
}


fn read_config(default_content: &str) -> Config {
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("{error}Failed to get executable file. {err}");
            exit(1);
        }
    };
    let path = exe.parent().unwrap();
    let contents = match fs::read_to_string(path.join("config.toml")) {
        Ok(contents) => contents,
        Err(_) => {
            eprintln!("{error}Could find the config file, creating new one");
            let data: Config = toml::from_str(default_content).unwrap();
            save_config(&data);
            default_content.to_owned()
        }
    };
    match toml::from_str(&contents) {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{error}Unable to load the config file. {}", err);
            exit(1);
        }
    }
}


fn save_config(data: &Config) {
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let contents = toml::to_string(data).unwrap();
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("{error}Failed to get executable file. {err}");
            exit(1);
        }
    };
    let path = exe.parent().unwrap();
    match fs::write(path.join("config.toml"), contents) {
        Ok(_) => {},
        Err(err) => eprintln!("{error}Failed to save the config file. {}", err)
    }
}


fn get_matches() -> Result<clap::ArgMatches, clap::Error> {
    return Command::new("plz")
        .about("plz is an alias manager to help you manage your games.")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("run")
                .about("Run an alias")
                .arg(
                    Arg::new("alias")
                        .help("The alias to run")
                        .required(true)
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
                .about("Fetch links from Game3rb and SteamRIP")
                .arg(
                    Arg::new("game")
                        .help("The game to fetch links for")
                        .required(true)
                )
        )
        .subcommand(
            Command::new("fetcha")
                .about("Fetch links from Game3rb and SteamRIP with reversed order")
                .arg(
                    Arg::new("game")
                        .help("The game to fetch links for")
                        .required(true)
                )
        )
    .try_get_matches();
}


async fn fetch_game_info(name: &str) -> Option<(String, Vec<String>)> {
    let v = AnsiColor::BrightYellow.on_default();
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
            eprintln!("{error}Error sending request: {}", err);
            return None;
        },
    };

    if res.status().as_u16() == 404 {
        return None;
    }

    let soup = Html::parse_document(&res.text().await.unwrap());
    let title = soup
        .select(&Selector::parse("h1.post-title").unwrap())
        .next()?
        .text()
        .collect::<String>()
        .replace("Download ", "")
        .replace(" + OnLine", " + Online")
        .trim().to_owned();

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

    println!("{success}Fetched Game3rb for `{v}{}{v:#}` in {v}{:.2}{v:#}s\n", name, perf.elapsed().as_secs_f64());
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


fn recursive_search(path: &str, folder_path: &str, config: &mut Config) -> io::Result<()> {
    let v = AnsiColor::BrightYellow.on_default();
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
            if !config.autoadd_ignore.contains(&file_path) && !config.aliases.values().any(|val| val == &file_path) {
                print!("Alias name for `{v}{}{v:#}` (enter to skip): ", filename);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let name = input.trim();
                
                if !name.is_empty() {
                    if config.aliases.contains_key(name) {
                        if user_input(format!("Overwrite alias `{v}{}{v:#}`? (y/n) ", name)) {
                            config.aliases.insert(name.to_string(), file_path);
                        }
                    } else {
                        config.aliases.insert(name.to_string(), file_path);
                    }
                } else {
                    config.autoadd_ignore.push(file_path);
                }
            }
        }
    } else if !folders.is_empty() {
        println!("No executables found in `{v}{}{v:#}`, going into subfolders", path);
        for folder in folders {
            let folder_path = folder.display().to_string();
            recursive_search(&folder_path, &folder_path, config)?;
        }
    }

    Ok(())
}


fn autoadd(config: &mut Config) -> io::Result<()> {
    let v = AnsiColor::BrightYellow.on_default();
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
            recursive_search(&file_name, &folder_path, config)?;
        } else if file_path.is_file() && file_name.ends_with(".exe") &&
                  !config.autoadd_ignore.contains(&file_path.display().to_string()) &&
                  !config.aliases.values().any(|val| val == &file_path.display().to_string()) {
                
                print!("Alias name for `{v}{}{v:#}` (enter to skip): ", file_name);
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let name = input.trim();
                
                if !name.is_empty() {
                    if config.aliases.contains_key(name) {
                        if user_input(format!("Overwrite alias `{v}{}{v:#}`? (y/n) ", name)) {
                            config.aliases.insert(name.to_string(), file_path.display().to_string());
                        }
                    } else {
                        config.aliases.insert(name.to_string(), file_path.display().to_string());
                    }
                } else {
                    config.autoadd_ignore.push(file_path.display().to_string());
                }
        }
    }

    save_config(&config);
    Ok(())
}


async fn fetch(name: &str) {
    let red = AnsiColor::BrightRed.on_default().bold();
    let error = format!("{red}error:{red:#} ");
    let bold = Style::new().bold();

    match fetch_game_info(name).await {
        Some((title, items)) => {
            println!("{bold}{}", title);
            println!("Game3rb Download links:{bold:#}");
            for item in items {
                println!("{item}");
            }
        }
        None => eprintln!("{error}Failed to fetch Game3rb")
    }
}

fn check_config(config: &mut Config) {
    let bold_yellow = AnsiColor::BrightYellow.on_default().bold();
    let v = AnsiColor::BrightYellow.on_default();
    let warning = format!("\n{bold_yellow}warning:{bold_yellow:#} ");
    let path = Path::new(&config.games_dir);
    
    if !path.exists() {
        eprint!("{warning}games_dir `{v}{}{v:#}` does not exist.", config.games_dir);
    } else if !path.is_dir() {
        eprint!("{warning}games_dir `{v}{}{v:#}` is not a directory.", config.games_dir);
    } else if !config.games_dir.contains(std::path::MAIN_SEPARATOR) {
        eprint!("{warning}games_dir `{v}{}{v:#}` doesn't use system's main separator ({}).", config.games_dir, MAIN_SEPARATOR);
    }

    for path in config.aliases.clone() {
        if !Path::new(&path.1).exists() {
            eprint!("{warning}Alias `{v}{}{v:#}` points to `{v}{}{v:#}` which does not exist.", path.0, path.1);
        } else if !Path::new(&path.1).is_file() {
            eprint!("{warning}Alias `{v}{}{v:#}` points to `{v}{}{v:#}` which is not a file.", path.0, path.1);
        }
    }
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
            let v = AnsiColor::BrightYellow.on_default().bold();
            return format!("\n{green}New version of plz available:{green:#}\n Current: {v}v{}{v:#}\n New version: {green}{}{green:#}", env!("CARGO_PKG_VERSION"), release.tag_name);
        }
    }
    String::new()
}


#[tokio::main]
async fn main() {
    let mut config: Config = read_config("games_dir = \"\"\nalternative_fetch = false\ncheck_for_updates = true\nautoadd_ignore = []\n[aliases]\n");
    let update_message = match config.check_for_updates {
        true => Some(check_for_updates()),
        false => None,
    };

    match get_matches() {
        Ok(matches) => {
            let red = AnsiColor::BrightRed.on_default().bold();
            let error = format!("{red}error:{red:#} ");
            let green = AnsiColor::BrightGreen.on_default().bold();
            let success = format!("{green}success:{green:#} ");
            let v = AnsiColor::BrightYellow.on_default();
            let bold = Style::new().bold();
            match matches.subcommand() {
                Some(("run", matches)) => {
                    let alias: &String = matches.get_one("alias").unwrap();
                    match config.aliases.get(alias) {
                        Some(path) => {
                            let path: &Path = Path::new(path);
                            let dir = match path.parent() {
                                Some(path) => path,
                                None => {
                                    eprintln!("{error}Path: {}. Failed to get the parent of path", path.display());
                                    exit(1);
                                }
                            };
                            match std::env::set_current_dir(dir) {
                                Ok(_) => {}
                                Err(err) => {
                                    eprintln!("{error}Path: {}. {}", path.display(), err);
                                    exit(1);
                                }
                            }
                            println!("{bold}Running:{bold:#} `{v}{}{v:#}`", path.display());
                            if let Err(err) = std::process::Command::new(path).status() {
                                eprintln!("{error}Failed to run alias `{v}{}{v:#}`: {}", alias, err);
                            }
                        },
                        None => eprintln!("{error}Alias `{v}{}{v:#}` not found", alias)
                    }
                }
                Some(("random", _)) => {
                    if config.aliases.len() == 0 {
                        eprintln!("{error}No aliases found");
                        exit(1);
                    }
                    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    let index = current_time % config.aliases.len() as u128;
                    if let Some((alias, value)) = config.aliases.iter().nth(index as usize) {
                        let path = Path::new(value);
                        let dir = match path.parent() {
                            Some(path) => path,
                            None => {
                                eprintln!("{error}Path: {}. Failed to get the parent of path", path.display());
                                exit(1);
                            }
                        };
                        match std::env::set_current_dir(dir) {
                            Ok(_) => {}
                            Err(err) => {
                                eprintln!("{error}Path: `{v}{}{v:#}`. {}", path.display(), err);
                                exit(1);
                            }
                        }
                        println!("{bold}Running:{bold:#} `{v}{}{v:#}`", path.display());
                        if let Err(err) = std::process::Command::new(value).status() {
                            eprintln!("{error}Failed to run alias `{v}{}{v:#}`: {}", alias, err);
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
                                    save_config(&config);
                                    println!("{success}Set value of check_for_updates to `{v}true{v:#}`");
                                } else if value.unwrap() == "false" {
                                    config.check_for_updates = false;
                                    save_config(&config);
                                    println!("{success}Set value of check_for_updates to `{v}false{v:#}`");
                                } else {
                                    eprintln!("{error}Value needs to be either `{v}false{v:#}` or `{v}true{v:#}`");
                                }
                            } else {
                                println!("Current value of check_for_updates is `{v}{}{v:#}`", config.check_for_updates);
                            }
                        }
                        Some(("games_dir", matches)) => {
                            let value: Option<&String> = matches.get_one("value");
                            if value.is_some() {
                                config.games_dir = value.unwrap().clone();
                                save_config(&config);
                                println!("{success}Set value of games_dir to `{v}{}{v:#}`", value.unwrap());
                            } else {
                                println!("Current value of games_dir is `{v}{}{v:#}`", config.games_dir);
                            }
                        }
                        _ => {
                            println!("{bold}Current config values:{bold:#}");
                            println!(" {bold}games_dir:{bold:#} `{v}{}{v:#}`", config.games_dir);
                            println!(" {bold}check_for_updates:{bold:#} `{v}{}{v:#}`", config.check_for_updates);
                        }
                    }
                }
                Some(("alias", matches)) => {
                    match matches.subcommand() {
                        Some(("add", matches)) => {
                            let alias: &String = matches.get_one("alias").unwrap();
                            let path: &String = matches.get_one("path").unwrap();
    
                            if config.aliases.contains_key(alias) {
                                if user_input(format!("Overwrite alias `{v}{}{v:#}`? (y/n) ", alias)) {
                                    config.aliases.insert(alias.to_string(), path.to_string());
                                    save_config(&config);
                                    println!("{success}Overwrote alias `{v}{}{v:#}`", alias);
                                }
                            } else {
                                config.aliases.insert(alias.to_string(), path.to_string());
                                save_config(&config);
                                println!("{success}Added alias `{v}{}{v:#}`", alias);
                            }
    
                        }
                        Some(("remove", matches)) => {
                            let alias: &String = matches.get_one("alias").unwrap();
                            if config.aliases.contains_key(alias) {
                                config.aliases.remove(alias);
                                save_config(&config);
                                println!("{success}Removed alias `{v}{}{v:#}`", alias);
                            } else {
                                println!("{error}Alias `{v}{}{v:#}` doesn't exist", alias);
                            }
                        }
                        Some(("list", _)) => {
                            let sorted = sort_by_key_length(config.aliases.clone());
                            let gray = AnsiColor::BrightBlack.on_default();
                            
                            println!("{bold}Aliases:");
                            for (alias, path) in sorted.iter() {
                                println!(" {bold}{}{bold:#} {gray}->{gray:#} {}", alias, path);
                            }
                        }
                        Some(("autoadd", _)) => {
                            match autoadd(&mut config) {
                                Ok(_) => {
                                    save_config(&config);
                                },
                                Err(err) => eprintln!("{error}{}", err)
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                Some(("fetch", matches)) => {
                    let game: &String = matches.get_one("game").unwrap();
                    fetch(game).await;
                }
                _ => unreachable!()
            }
        }
        Err(err) => err.print().unwrap()
    }
    check_config(&mut config);
    match update_message {
        Some(future) => {
            println!("{}", future.await);
        }
        None => {}
    }
}
