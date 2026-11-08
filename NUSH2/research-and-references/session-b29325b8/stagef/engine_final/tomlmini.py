import json, re
def load(path):
    out, cur = [], None
    for line in open(path, encoding="utf-8"):
        s = line.strip()
        if s == "[[param]]":
            cur = {}; out.append(cur); continue
        if cur is None or not s or s.startswith("#"): continue
        k, _, v = s.partition("=")
        k, v = k.strip(), v.strip()
        if v.startswith('"'):
            cur[k] = json.loads(v)
        else:
            cur[k] = v
    return out
