"""Minimal reader for engine/data/parameters.toml ([[param]] blocks with scalar keys)."""
import re

def load(path):
    t = open(path, encoding="utf-8").read()
    out = {}
    for b in re.split(r"\n(?=\[\[)", t):
        m = re.search(r'^id\s*=\s*"([^"]+)"', b, re.M)
        if not m:
            continue
        d = {}
        for line in b.splitlines():
            mm = re.match(r'^(\w+)\s*=\s*(.*)$', line)
            if not mm:
                continue
            k, v = mm.group(1), mm.group(2).strip()
            if v.startswith('"'):
                d[k] = v.strip('"')
            else:
                try:
                    d[k] = float(v)
                except ValueError:
                    d[k] = v
        out[m.group(1)] = d
    return out
