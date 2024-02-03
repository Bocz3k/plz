import os
import shutil

def main():
    os.system('pyinstaller --noconfirm --onedir --console --icon "D:/Programming/Python/plz/plz.ico" --add-data "D:/Programming/Python/plz/theliblib.py;." --hidden-import "tomlkit" --hidden-import "requests" --hidden-import "bs4"  "D:/Programming/Python/plz/plz.py"')
    os.remove('plz.spec')
    if os.path.exists('bin/plz.exe'):
        os.remove('bin/plz.exe')
        shutil.rmtree('bin/_internal')
    shutil.move('dist/plz/plz.exe', 'bin')
    shutil.move('dist/plz/_internal', 'bin')
    os.removedirs('dist/plz')
    print("Done!")


if __name__ == '__main__':
    main()
