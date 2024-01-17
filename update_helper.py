import shutil
import os
from theliblib import get_dir


def main():
    for item in os.listdir('update'):
        source = os.path.join('update', item)
        destination = os.path.join(get_dir(), item)
        if os.path.exists(destination):
            os.remove(destination)
        shutil.move(source, destination)
    os.remove('update.zip')
    shutil.rmtree('update')


if __name__ == '__main__':
    main()
