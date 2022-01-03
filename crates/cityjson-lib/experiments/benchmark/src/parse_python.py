import json
from sys import argv

with open(argv[1], mode="r") as fo:
    json.load(fo)