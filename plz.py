import zipfile
import sys
import theliblib as tl
from theliblib import SubCmd, ArgType
import random
import os
import shutil
import requests
import toml
from bs4 import BeautifulSoup

VERSION = 'v0.1-beta'


def helpfunc(topic: str = None):
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
"""


try:
    with open(tl.get_dir() + 'config.toml') as f:
        cfg = toml.load(f)
    with open(tl.get_dir() + 'aliases.toml') as f:
       aliases = toml.load(f)
except FileNotFoundError:
    with open(tl.get_dir() + 'config.toml', 'w') as f:
        toml.dump({'runin': '', 'clear_runin': False}, f)
    with open(tl.get_dir() + 'aliases.toml', 'w') as f:
        f.write('')
    cfg = {'runin': None, 'clear_runin': False}
    aliases = {}


def save_config():
    with open(tl.get_dir() + 'config.toml', 'w') as f:
        toml.dump(cfg, f)


def save_aliases():
    with open(tl.get_dir() + 'aliases.toml', 'w') as f:
        toml.dump(aliases, f)


def run(alias: str):
    try:
        os.system(f'"{aliases[alias]}"')
        if cfg['clear_runin']:
            try: shutil.rmtree(cfg['runin'])
            except PermissionError: pass
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


def sub_fetch(name: str):
    res = requests.get("https://steamrip.com/" + name)
    soup = BeautifulSoup(res.text, "html.parser")
    items = soup.find_all("p", {"style": "text-align: center;"})
    if not items:
        return "Game not found"
    items.pop(0)
    try:  # Try old game compatibility
        info_list = soup.find('div', {"class": "plus tie-list-shortcode"}).find('ul').find('li').find('ul').find_all('li')
    except AttributeError:
        info_list = soup.find('div', {"class": "plus tie-list-shortcode"}).find('ul').find_all('li')
    
    size = info_list[3].get_text()
    ver: str = info_list[5].get_text()
    t = ver.partition(' | ')
    print(t[0].encode('ascii', 'ignore').decode())  # Avoid getting UTF-8 Error in runner
    print(size)
    print("Download links:")
    for elem in items:
        print(f"- {elem.find("span").text.rstrip()}: https:{elem.find('a').get('href')}")


# there's a todo in theliblib.py that needs to be done first
# def sub_config(attribute_name: str, value: str | bool):
#     cfg.update({attribute_name: value})
#     save_config()


def check_for_updates():
    response = requests.get("https://api.github.com/repos/Bocz3k/plz/releases/latest")
    if response.status_code == 200:
        latest_version = response.json()['tag_name']
        if latest_version != VERSION:
            return f'New version available: {latest_version}\n' \
                   f'Download the update from {response.json()["html_url"]}'


def main():
    cli = tl.CLITool()
    cli.add_subcmd(SubCmd("random", sub_random))
    cli.add_subcmd(SubCmd("edit", sub_edit))
    cli.add_subcmd(SubCmd("run", run, [ArgType(str, options=aliases.keys())]))
    cli.add_subcmd(SubCmd("fetch", sub_fetch, [ArgType(str)]))
    # cli.add_subcmd(SubCmd("config", sub_config, [ArgType(str, options=['runin', 'clear_runin']), ArgType(str | bool)]))
    cli.add_subcmd(SubCmd("alias", subcmds=[
        SubCmd("list", sub_alias_list),
        SubCmd("add", sub_alias_add, [ArgType(str), ArgType(str)]),
        SubCmd("remove", sub_alias_remove, [ArgType(str, options=aliases.keys())])
    ]))
    os.chdir(runin) if (runin := cfg['runin']) else ''
    cli.run(sys.argv, helpfunc, check_for_updates)


if __name__ == '__main__':
    main()
