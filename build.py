import os
import shutil

def main():
    os.system('pyinstaller --noconfirm --onedir --console --icon "plz.ico" --add-data "theliblib.py;." --hidden-import "tomlkit" --hidden-import "requests" --hidden-import "bs4" "plz.py"')
    os.remove('plz.spec')
    if os.path.exists('bin/plz.exe'):
        os.remove('bin/plz.exe')
        shutil.rmtree('bin/_internal')
    shutil.move('dist/plz/plz.exe', 'bin')
    shutil.move('dist/plz/_internal', 'bin')
    os.removedirs('dist/plz')
    print("Done!")
    os.system('explorer bin')


if __name__ == '__main__':
    main()
