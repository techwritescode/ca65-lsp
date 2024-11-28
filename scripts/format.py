from itertools import batched
import typing
import json

with open("./instructions/6502.txt", "r") as file:
    lines = file.read().splitlines()
    defs = typing.cast(list[tuple[str, str]], list(batched(lines,2)))
    docs = dict(defs)
    print(json.dumps(docs))
    
