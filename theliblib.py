import __main__
from platform import system as platform
# TODO: Handle union types in ArgType.test


def to_seconds(human_input: str) -> int:
    """
    Converts a human input into seconds
    Example: 1y 2mo 3d 4h 5m 6s
    """
    time = str(human_input).split()
    years, months, days, hours, minutes, seconds = 0, 0, 0, 0, 0, 0
    for t in time:
        if 'y' in t:
            years = int(t[:-1])
        elif 'mo' in t:
            months = int(t[:-2])
        elif 'd' in t:
            days = int(t[:-1])
        elif 'h' in t:
            hours = int(t[:-1])
        elif 'm' in t:
            minutes = int(t[:-1])
        elif 's' in t:
            seconds = int(t[:-1])
        seconds = seconds + (minutes * 60) + (hours * 3600) + (days * 86400) + (months * 2419200) + (years * 29030400)
    return seconds


def get_dir():
    """
    Get path to folder where your script is located
    """
    if platform() == "Windows":
        fp = __main__.__file__.split('\\')
        fp.pop()
        return str(fp).replace("'", "").replace(', ', '\\').replace('[', '').replace(']', '') + "\\"
    elif platform() == "Linux":
        fp = __main__.__file__.split('/')
        fp.pop()
        return str(fp).replace("'", "").replace(', ', '/').replace('[', '').replace(']', '') + "/"
    else:
        raise NotImplementedError("Function is not supported on " + platform())


def cinput(prompt: str, requested_type: type = None, default=''):
    """
    Asks user with the `prompt`, and if `requested_type` is not none, tries to convert the user's
    input to the `requested_type`\n
    `requested_type` - requested type of output (int, str, etc.)
    """
    while True:
        try:
            user_input = input(prompt)
            if user_input == '':
                return default
            if not requested_type is None:
                user_input = requested_type(user_input)
            break
        except ValueError:
            return None
    return user_input


def mass_replace(s: str, to_replace: list[str], replace_to: str):
    for r in to_replace:
        s = s.replace(r, replace_to)
    return s


class ArgType:
    def __init__(self, _type: type, required: bool = True, options: list = None, continuos: bool = False) -> None:
        self.type = _type
        self.required = required
        self.continuos = continuos
        self.options = options or []

    def test(self, input: str):
        verdict = 'Wrong type'
        try:
            input = self.type(input)
            verdict = f'Wrong option (Options: {mass_replace(str(self.options), ['[', ']', "'", '"'], '')})'
            if self.options:
                if input not in self.options:
                    raise TypeError
        except TypeError:
            return verdict
        return 'pass'


class SubCmd:
    def __init__(self, name: str, func=None, arguments: list[ArgType] = None, subcmds: list = None) -> None:
        self.id = name
        self.func = func
        self.subcmds = subcmds or []
        self.arguments = arguments or []
        self.path = None

    def _gen_path(self, subcmd):
        for sub in subcmd.subcmds:
            sub.path = subcmd.path + ' ' + sub.id
            if sub.subcmds:
                self._gen_path(sub)

    def run(self, args):
        if self.func:
            res = self.func(*args)
            if res is not None:
                print(res)
                return res


class CLITool:
    def __init__(self) -> None:
        self.tree = SubCmd("Tree")
        self.add_tree = False

    def add_subcmd(self, subcmd: SubCmd) -> None:
        subcmd.path = subcmd.id
        if subcmd.subcmds:
            subcmd._gen_path(subcmd)
        self.tree.subcmds.append(subcmd)

    def into_subcmd(self) -> None:
        self.add_tree = True

    def add_arg(self, arg_type: ArgType) -> None:
        if not self.add_tree:
            raise AttributeError("CLITool instance hasn't been turned into a subcommand.")
        self.tree.arguments.append(arg_type)

    def _get_stack(self, args: list[str]) -> list[SubCmd, str]:
        if self.add_tree:
            args.insert(0, self.tree)
        stack = []
        args_left = []
        for arg in args:
            if len(args_left) > 0:
                if args_left[0].continuos:
                    i = args.index(arg)
                    z = mass_replace(str(args[i:]), ['[', ']', "'"], '')
                    z = z.replace(', ', ' ')
                    stack.append(z)
                    args_left.pop(0)
                    break
                if not ((verdict := args_left[0].test(arg)) == 'pass'):
                    if self.helpfunc:
                        stack.reverse()
                        for item in enumerate(stack):
                            if isinstance(item, SubCmd):
                                print(r) if (r := self.helpfunc(item.path)) else 0
                    print(f"\n`{arg}`: {verdict}")
                    return
                args_left.pop(0)
                stack.append(arg)
            elif self._is_subcmd(arg):
                for subcmd in self._get_all_subcmds(self.tree):
                    if subcmd.id == arg:
                        stack.append(subcmd)
                        args_left = subcmd.arguments.copy()
                        break
            else:
                if self.helpfunc:
                    print(r) if (r := self.helpfunc("")) else 0
                print("Wrong command: " + arg)
                return
        if args_left:
            stack.reverse()
            if self.helpfunc:
                for item in stack:
                    if isinstance(item, SubCmd):
                        print(r) if (r := self.helpfunc(item.path)) else 0
            print("More arguments needed")
            return
        return stack


    def _is_subcmd(self, string: str) -> bool:
        for subcmd in self._get_all_subcmds(self.tree):
            if subcmd.id == string:
                return True
        return False

    def _get_all_subcmds(self, subcmd: SubCmd) -> list[SubCmd]:
        subcmds = []
        for _ in range(len(subcmd.subcmds)):
            for sub in subcmd.subcmds:
                subcmds.append(sub)
                subcmds.extend(self._get_all_subcmds(sub))
        return subcmds

    def run(self, argv: list[str], helpfunc = None, always_run = None) -> None:
        self.helpfunc = helpfunc
        if helpfunc:
            self.add_subcmd(SubCmd("help", helpfunc, [ArgType(str, continuos=True)]))
        stack = self._get_stack(argv[1:])
        if not stack:
            return
        stack.reverse()
        for i, item in enumerate(stack):
            if isinstance(item, SubCmd):
                r = stack[:i]
                r.reverse()
                item.run(r)
                if always_run:
                    print(z) if (z := always_run()) else 0
