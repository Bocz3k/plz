from theliblib import SubCmd, ArgType
from bs4 import BeautifulSoup
import theliblib as tl
import sys
import random
import os
import requests
import toml
import time
import logging


VERSION = 'v0.1.1-beta'
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
       plz alias list - Prints a list of all aliases.
       plz alias add <name> <point> - Adds an alias to the list.
       plz alias remove <name> - Removes an alias from the list.
       plz alias autoadd - Adds all games in the games folder to the list.
"""


def fix_file(path: str):
    try:
        with open(f'{tl.get_dir()}\\..\\{path}.toml') as f:
            result = toml.load(f)
    except FileNotFoundError:
        logging.warning(f'No {path} file found. Creating one')
        with open(f'{tl.get_dir()}\\..\\{path}.toml', 'w') as f:
            f.write('')
        with open(f'{tl.get_dir()}\\..\\{path}.toml') as f:
            result = toml.load(f)
    except toml.TomlDecodeError:
        logging.warning(f'{path} file is corrupted')
        overwrite = tl.cinput("Do you want to overwrite it with nothing? (y/n) ", default="n", options=["y", "n"])
        if overwrite == "y":
            with open(f'{tl.get_dir()}\\..\\{path}.toml', 'w') as f:
                f.write('')
            with open(f'{tl.get_dir()}\\..\\{path}.toml') as f:
                result = toml.load(f)
        else:
            return None
    return result


def fix_config():
    # TODO: Add warning when alias' path doesn't exist
    # TODO: Continue switching to toml instead of tomlkit
    global cfg, aliases, autoadd_ignore
    try:
        with open(tl.get_dir() + '\\..\\config.toml') as f:
            cfg = toml.load(f)
    except FileNotFoundError:
        print('WARNING: No config file found. Creating one')
        cfg = {'games_dir': "", 'log_level': "WARNING", 'fetch_sites': [], 'info_site': "game3rb"}
        save_config()
        main()
        return True
    except toml.TomlDecodeError:
        print('WARNING: Config file is corrupted')
        overwrite = tl.cinput("Do you want to overwrite it with nothing? (y/n) ", default="n", options=["y", "n"])
        if overwrite == "y":
            with open(f'{tl.get_dir()}\\..\\{path}.toml', 'w') as f:
                f.write('')
            with open(f'{tl.get_dir()}\\..\\{path}.toml') as f:
                cfg = toml.load(f)
        else:
            return True
    
    if (
        cfg['log_level'] != 'DEBUG' and
        cfg['log_level'] != 'INFO' and
        cfg['log_level'] != 'WARNING' and
        cfg['log_level'] != 'ERROR' and
        cfg['log_level'] != 'NONE'
    ):
        print("WARNING: log_level must be one of the following: DEBUG, INFO, WARNING, ERROR, NONE. Defaulting to WARNING")
        cfg['log_level'] = 'WARNING'

    if cfg['log_level'] == "NONE":
        logging.disable()
    else:
        if cfg['log_level'] == "DEBUG":
            logging.basicConfig(level=logging.DEBUG,
                                format='%(relativeCreated)d: %(funcName)s | %(levelname)s: %(message)s')
        else:
            logging.basicConfig(level=eval("logging." + cfg['log_level'], globals()),
                                format='%(relativeCreated)d | %(levelname)s: %(message)s')

    aliases = fix_file("aliases")
    if aliases is None:
        return True
    autoadd_ignore = fix_file("autoadd-ignore")
    if autoadd_ignore is None:
        return True

    for alias, path in aliases.items():
        if path.find('/') != -1:
            aliases[alias] = path.replace('/', '\\')
            logging.warning(f'Found "/" in alias {alias}. Replacing with "\\"')

    for i, path in enumerate(autoadd_ignore):
        if path.find('/') != -1:
            autoadd_ignore[i] = path.replace('/', '\\')
            logging.warning(f'Found "/" in autoadd-ignore. Replacing with "\\"')
    
    if isinstance(cfg['games_dir'], str):
        if cfg['games_dir'].find('/') != -1:
            logging.warning('Found "/" in games_dir. Replacing with "\\"')
            cfg['games_dir'] = cfg['games_dir'].replace('/', '\\')
        elif cfg['games_dir'] == '':
            logging.warning('games_dir is empty, plz alias autoadd won\'t work. You\'ll need to fix it yourself')
    else:
        logging.warning('games_dir must be a string. Defaulting to ""')
        cfg['games_dir'] = ''
    
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
    save_aliases()
    save_config()


def save_config():
    logging.debug("Saving config")
    with open(tl.get_dir() + '\\..\\config.toml', 'w') as f:
        toml.dump(cfg, f)


def save_aliases():
    logging.info('Saving aliases')
    with open(tl.get_dir() + '\\..\\aliases.toml', 'w') as f:
        toml.dump(aliases, f)


def run(alias: str):
    idx = aliases[alias].rfind('\\')
    os.chdir(aliases[alias][:idx])
    try:
        os.system(f'"{aliases[alias]}"')
    except KeyError:
        return f'No alias found named "{alias}"'


def sub_random():
    key = random.choice([k for k in aliases.keys()])
    run(key)


def sub_alias_list():
    max_len = max(len(game) for game in aliases.keys()) + 1
    for alias, path in aliases.items():
        print(f'{alias.ljust(max_len)}{path}')


def sub_alias_add(alias: str, point: str):
    if alias in aliases.keys():
        overwrite = tl.cinput(f'Alias {alias} already exists. Overwrite? (y/n): ', default='n', options=['y', 'n'])
        if overwrite == 'y':
            aliases.update({alias: point})
            save_aliases()


def sub_alias_remove(alias: str):
    aliases.pop(alias)
    save_aliases()


def recursive_search(path: str, folder_path: str):
    executables = []
    folders = []
    for file in os.listdir(folder_path):
        isdir = os.path.isdir(os.path.join(folder_path, file))
        if not isdir and file.endswith('.exe') and not file in EXECUTABLE_BLACKLIST:
            executables.append(os.path.join(folder_path, file))
        elif isdir:
            folders.append(os.path.join(folder_path, file))
    
    if executables:
        for executable_file in executables:
            if not os.path.join(folder_path, executable_file) in aliases.values():
                name = input(f'Alias name for {executable_file} (enter to skip): ')
                if name:
                    sub_alias_add(name, os.path.join(folder_path, executable_file))
                else:
                    autoadd_ignore['ignore'].append(os.path.join(folder_path, executable_file))
                    with open(f'{tl.get_dir()}\\..\\autoadd-ignore.toml', 'w') as f:
                        toml.dump(autoadd_ignore, f)
    else:
        if folders:
            logging.info(f"No executables found in {path}, going into subfolders")
            for folder in folders:
                recursive_search(folder, os.path.join(folder_path, folder))


def sub_alias_autoadd():
    if cfg['games_dir'] == '':
        return "games_dir is empty, plz alias autoadd won't work. You'll need to fill it yourself"
    for path in os.listdir(cfg['games_dir']):
        if os.path.isdir(folder_path := os.path.join(cfg['games_dir'], path)):
            recursive_search(path, folder_path)
        else:
            if path.endswith('.exe'):
                name = input(f'Alias name for {path} (enter to skip): ')
                if name:
                    sub_alias_add(name, os.path.join(folder_path, path))


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
        if host.find('www.') != -1:
            idx += 4
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
    response = requests.get("https://api.github.com/repos/Bocz3k/plz/releases/latest")
    if response.status_code == 200:
        latest_version = response.json()['tag_name']
        if latest_version != VERSION:
            return f'New version available: {latest_version}\n' \
                   f'Download the update from {response.json()["html_url"]}'


def main():
    if fix_config():
        return

    cli = tl.CLITool()
    cli.add_subcmd(SubCmd("random", sub_random))
    cli.add_subcmd(SubCmd("run", run, [ArgType(str, options=aliases.keys())]))
    cli.add_subcmd(SubCmd("fetch", sub_fetch, [ArgType(str)]))
    cli.add_subcmd(SubCmd("alias", subcmds=[
        SubCmd("list", sub_alias_list),
        SubCmd("add", sub_alias_add, [ArgType(str), ArgType(str)]),
        SubCmd("remove", sub_alias_remove, [ArgType(str, options=aliases.keys())]),
        SubCmd("autoadd", sub_alias_autoadd),
    ]))
    cli.run(sys.argv, helpfunc, check_for_updates)


if __name__ == '__main__':
    main()
