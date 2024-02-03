import sys
import theliblib as tl
from theliblib import SubCmd, ArgType
import random
import os
import shutil
import requests
import tomlkit
import time
import logging
from bs4 import BeautifulSoup

VERSION = 'v0.1.1-beta-dev'
EXECUTABLE_BLACKLIST = [
    "unins000.exe",
    "UnityCrashHandler64.exe",
    "UnityCrashHandler32.exe"
]


def helpfunc(topic: str = None):
    logging.debug(f'Help topic: {topic}')
    if topic == 'run':
        return "Usage: plz run <alias>\nRuns the file binded to the alias."
    elif topic == 'random':
        return "Usage: plz random\nSelects a random game and runs it."
    elif topic == 'fetch':
        return "Usage: plz fetch <game>\nFetches download links for that game."
    elif topic == 'edit':
        return "Usage: plz edit\nOpens VSCode where this CLI tool is."
    elif topic == 'alias list':
        return "Usage: plz alias list\nPrints a list of all aliases."
    elif topic == 'alias add':
        return "Usage: plz alias add <name> <point>\nAdds an alias to the list."
    elif topic == 'alias remove':
        return "Usage: plz alias remove <name>\nRemoves an alias from the list."
    elif topic == 'alias autoadd':
        return "Usage: plz alias autoadd\nAdds all games in the games folder to the list."
    return \
"""
Usage: plz help - Shows this menu.
       plz run <alias> - Runs the file binded to the alias.
       plz random - Selects a random game and runs it.
       plz fetch <game> - Fetches download links for that game.
       plz edit - Opens VSCode where this CLI tool is.
       plz alias list - Prints a list of all aliases.
       plz alias add <name> <point> - Adds an alias to the list.
       plz alias remove <name> - Removes an alias from the list.
       plz alias autoadd - Adds all games in the games folder to the list.
"""



def fix_config():
    global cfg, aliases
    try:
        with open(tl.get_dir() + 'config.toml') as f:
            cfg = tomlkit.load(f)
    except FileNotFoundError:
        print('WARNING: No config file found. Creating one')
        cfg = {'runin': "", 'clear_runin': False, 'games_dir': "", 'log_level': "WARNING", 'fetch_sites': [], 'info_site': "game3rb"}
        save_config()
        main()
        return True
    
    try:
        with open(tl.get_dir() + 'aliases.toml') as f:
            aliases = tomlkit.load(f)
    except FileNotFoundError:
        logging.warning('No aliases file found. Creating one')
        with open(tl.get_dir() + 'aliases.toml', 'w') as f:
            f.write('')
        aliases = {}

    if isinstance(cfg['runin'], str):
        if cfg['runin'].find('/') != -1:
            logging.warning('Found "/" in runin. Replacing with "\\"')
            cfg['runin'] = cfg['runin'].replace('/', '\\')
    else:
        logging.warning('runin must be a string. Defaulting to ""')
        cfg['runin'] = ''
    
    if isinstance(cfg['games_dir'], str):
        if cfg['games_dir'].find('/') != -1:
            logging.warning('Found "/" in games_dir. Replacing with "\\"')
            cfg['games_dir'] = cfg['games_dir'].replace('/', '\\')
        elif cfg['games_dir'] == '':
            logging.warning('games_dir is empty, plz alias autoadd won\'t work. You\'ll need to fix it yourself')
    else:
        logging.warning('games_dir must be a string. Defaulting to ""')
        cfg['games_dir'] = ''
    
    if cfg['clear_runin'] != True and cfg['clear_runin'] != False:
        logging.warning("clear_runin must be a boolean. Defaulting to false")
        cfg['clear_runin'] = False
    
    if (
        cfg['log_level'] != 'DEBUG' and
        cfg['log_level'] != 'INFO' and
        cfg['log_level'] != 'WARNING' and
        cfg['log_level'] != 'ERROR' and
        cfg['log_level'] != 'NONE'
    ):
        logging.warning("log_level must be one of the following: DEBUG, INFO, WARNING, ERROR, NONE. Defaulting to WARNING")
        cfg['log_level'] = 'WARNING'
    
    if isinstance(cfg['fetch_sites'], list):
        for site in cfg['fetch_sites']:
            if site != "steamrip" and site != "game3rb":
                logging.warning(f'fetch_sites must be a list containing "steamrip", "game3rb" or nothing. Defaulting to []')
                cfg['fetch_sites'] = []
    else:
        logging.warning(f'fetch_sites must be a list. Defaulting to []')
        cfg['fetch_sites'] = []
    
    if cfg['info_site'] != "game3rb" and cfg['info_site'] != "steamrip":
        logging.warning(f'info_site must be "game3rb" or "steamrip". Defaulting to "game3rb"')
        cfg['info_site'] = "game3rb"

    logging.info("Saving config")
    save_config()


def save_config():
    with open(tl.get_dir() + 'config.toml', 'w') as f:
        toml = tomlkit.document()
        toml.add(tomlkit.comment('Directory to run the game in | default: "" (make it "" for current working directory)'))
        toml.add('runin', cfg['runin'])
        toml.add(tomlkit.comment('Clear the runin folder after the game is run | default: false (true/false)'))
        toml.add('clear_runin', cfg['clear_runin'])
        toml.add(tomlkit.comment('Directory where the game folders are located | default: "" (if it\'s empty plz alias autoadd won\'t work)'))
        toml.add('games_dir', cfg['games_dir'])
        toml.add(tomlkit.comment('Logging level | default: WARNING (DEBUG/INFO/WARNING/ERROR/NONE)'))
        toml.add('log_level', cfg['log_level'])
        toml.add(tomlkit.comment('Sites to fetch game info from | default: [] (game3rb, steamrip)'))
        toml.add('fetch_sites', cfg['fetch_sites'])
        toml.add(tomlkit.comment('Game info site | default: game3rb (game3rb, steamrip)'))
        toml.add('info_site', cfg['info_site'])
        tomlkit.dump(toml, f)


def save_aliases():
    logging.info('Saving aliases')
    with open(tl.get_dir() + 'aliases.toml', 'w') as f:
        tomlkit.dump(aliases, f)


def run(alias: str):
    try:
        os.system(f'"{aliases[alias]}"')
        if cfg['clear_runin']:
            try: shutil.rmtree(cfg['runin'])
            except PermissionError: logging.debug('Could not delete runin folder due to permission error')
    except KeyError:
        return f'No alias found named "{alias}"'


def sub_random():
    key = random.choice([k for k in aliases.keys()])
    run(key)


def sub_edit():
    os.system('code ' + tl.get_dir())


def sub_alias_list():
    max_len = max(len(game) for game in aliases.keys()) + 1
    for alias, path in aliases.items():
        print(f'{alias.ljust(max_len)}{path}')


def sub_alias_add(alias: str, point: str):
    aliases.update({alias: point})
    save_aliases()


def sub_alias_remove(alias: str):
    aliases.pop(alias)
    save_aliases()


def sub_alias_autoadd():
    logging.info('Auto adding games')
    for path in os.listdir(cfg['games_dir']):
        logging.debug(f'Checking {path}')
        if os.path.isdir(folder_path := os.path.join(cfg['games_dir'], path)):
            logging.debug(f'Checking {path} for executables')
            executables = []
            for file in os.listdir(folder_path):
                if file.endswith('.exe') and not file in EXECUTABLE_BLACKLIST:
                    executables.append(os.path.join(folder_path, file))
            for executable in executables:
                logging.debug("exec: " + os.path.join(folder_path, executable))
                if os.path.join(folder_path, executable) in aliases.values():
                    logging.debug(f'Skipping {path} because it is already in the aliases list')
                    continue
            if executables:
                if len(executables) > 1:
                    for i, executable in enumerate(executables):
                        print(f"[{i}]: {executable}")
                    executable_file = executables[tl.cinput("Which executable should be used? ", int)]
                else:
                    executable_file = executables[0]
                name = input(f'Alias name for {executable_file}: ')
                sub_alias_add(name, os.path.join(folder_path, executable_file))
            else:
                logging.debug(f'Skipping {path} because it does not contain any executables')
        else:
            logging.debug(f'Skipping {path} because it is not a folder')
    save_aliases()


def fetch_steamrip(name: str):
    perf = time.perf_counter()
    res = requests.get("https://steamrip.com/" + name)
    if res.status_code == 404:
        return
    soup = BeautifulSoup(res.text, "html.parser")
    name = soup.find('h1', {"class": "post-title"}).get_text()
    index = name.find(' Free Download')
    name = name[:index]
    items = soup.find_all("p", {"style": "text-align: center;"})
    items.pop(0)
    items = [f"{item.find("span").text.rstrip()}: https:{item.find('a').get('href')}" for item in items]
    try:  # Try old game compatibility
        info_list = soup.find('div', {"class": "plus tie-list-shortcode"}).find('ul').find('li').find('ul').find_all('li')
    except AttributeError:
        info_list = soup.find('div', {"class": "plus tie-list-shortcode"}).find('ul').find_all('li')
    
    size = info_list[3].get_text()
    ver = info_list[5].get_text().replace("Version: ", "")
    t = ver.partition(' | ')
    logging.info(f'Fetched SteamRIP for {name} in {time.perf_counter() - perf:2f}s')
    return t[0].encode('ascii', 'ignore').decode(), size, name, items


def fetch_game3rb(name: str):
    perf = time.perf_counter()
    res = requests.get("https://game3rb.com/" + name)
    if res.status_code == 404:
        return
    soup = BeautifulSoup(res.text, "html.parser")
    name = name.lower()
    game_name = soup.find('h1', {"class": "post-title"}).get_text()
    
    version = ""
    game_name = game_name[9:].lower()
    size = soup.find("strong").parent.get_text()
    size = size.replace("RELEASE", "") \
               .replace("SIZE", "Size:") \
               .replace("::", ":") \
               .strip() 

    if game_name.find(name) != -1:
        version = game_name[len(name) + 1:]
        game_name = name.title()
    else:
        print("Game name not found")
    version = version.replace(' + online', '')
    
    item = soup.find("a", {"id": "download-link"})
    res = requests.get(item.get('href'))
    soup = BeautifulSoup(res.text, "html.parser")
    links = soup.find("ol").find_all("li")
    items = []
    for link in links:
        host = link.find('a').get('href')
        idx = host.find('://') + 3
        slash = host.find('/', idx)
        name = host[idx:slash]
        subdomain = name.find('.')
        items.append(f"{name[:subdomain].title()}: {host}")
    
    logging.info(f'Fetched Game3rb for {name} in {time.perf_counter() - perf:2f}s')
    return version, size, game_name, items


def sub_fetch(name: str):
    perf = time.perf_counter()
    try:
        if cfg['info_site'] == "steamrip":
            version, size, game_name, items = fetch_steamrip(name)
        else:
            version, size, game_name, items = fetch_game3rb(name)
    except TypeError:
        return f'No game found named "{name}"'
    
    for site in cfg['fetch_sites']:
        if site == "steamrip":
            _, _, _, site_items = fetch_steamrip(name)
        else:
            _, _, _, site_items = fetch_game3rb(name)
        items.extend(site_items)
    logging.info(f'Fetched for {game_name} in {time.perf_counter() - perf:2f}s')
    print("Name:", game_name)
    print("Version:", version) if version else 0
    print(size)
    print("Download links:")
    for item in items:
        print(f"- {item}")


def check_for_updates():
    if VERSION.endswith('-dev'):
        return
    response = requests.get("https://api.github.com/repos/Bocz3k/plz/releases/latest")
    if response.status_code == 200:
        latest_version = response.json()['tag_name']
        if latest_version != VERSION:
            return f'New version available: {latest_version}\n' \
                   f'Download the update from {response.json()["html_url"]}'


def main():
    if fix_config():
        return

    if cfg['log_level'] == "NONE":
        logging.disable()
    else:
        if cfg['log_level'] == "DEBUG":
            logging.basicConfig(level=eval("logging." + cfg['log_level'], globals()),
                                format='%(relativeCreated)d: %(name)s/%(funcName)s | %(levelname)s: %(message)s')
        else:
            logging.basicConfig(level=eval("logging." + cfg['log_level'], globals()),
                                format='%(relativeCreated)d: %(name)s | %(levelname)s: %(message)s')

    cli = tl.CLITool()
    cli.add_subcmd(SubCmd("random", sub_random))
    cli.add_subcmd(SubCmd("edit", sub_edit))
    cli.add_subcmd(SubCmd("run", run, [ArgType(str, options=aliases.keys())]))
    cli.add_subcmd(SubCmd("fetch", sub_fetch, [ArgType(str)]))
    cli.add_subcmd(SubCmd("alias", subcmds=[
        SubCmd("list", sub_alias_list),
        SubCmd("add", sub_alias_add, [ArgType(str), ArgType(str)]),
        SubCmd("remove", sub_alias_remove, [ArgType(str, options=aliases.keys())]),
        SubCmd("autoadd", sub_alias_autoadd),
    ]))
    os.chdir(runin) if (runin := cfg['runin']) else 0
    cli.run(sys.argv, helpfunc, check_for_updates)


if __name__ == '__main__':
    main()
