import simdjson
from sys import argv

with open(argv[1], mode="r") as fo:
    simdjson.load(fo)