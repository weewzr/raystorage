import json, sys
old = json.load(open(sys.argv[1], encoding="utf-8"))["tokens"]
new = json.load(open(sys.argv[2], encoding="utf-8"))["tokens"]
head = json.load(open(sys.argv[3], encoding="utf-8"))["tokens"]
derived_old = [k for k, v in old.items() if "resolve_numbers" in str(v.get("source", ""))]
print("old derived in resolve_numbers:", len(derived_old))
for k in derived_old:
    if k not in new:
        print("NOT IN NEW numbers.json:", k, old[k]["text"]); continue
    o, n = old[k], new[k]
    if o["text"] != n["text"] or o["table"] != n["table"]:
        print("TEXT DIFF", k, repr(o["text"]), repr(n["text"]), "|", repr(o["table"]), repr(n["table"]), o["unit"], n["unit"])
    elif o["unit"] != n["unit"]:
        print("UNIT DIFF", k, o["unit"], n["unit"])
for k in new:
    if k not in head and k not in old:
        print("NEW ONLY:", k, new[k]["text"], new[k]["unit"])
