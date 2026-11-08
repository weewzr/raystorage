import json, sys
d = json.load(open(sys.argv[1], encoding="utf-8"))["reply"]
lo, hi = float(sys.argv[2]), float(sys.argv[3])
def rec(v, path):
    if isinstance(v, dict):
        for k, x in v.items():
            rec(x, f"{path}.{k}" if path else k)
    elif isinstance(v, list):
        for i, x in enumerate(v):
            key = x.get("id") if isinstance(x, dict) and "id" in x else str(i)
            rec(x, f"{path}.{key}")
    elif isinstance(v, (int, float)) and not isinstance(v, bool):
        if lo <= v <= hi:
            print(path, v)
rec(d, "")
