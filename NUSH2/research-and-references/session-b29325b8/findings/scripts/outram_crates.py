"""Static, read-only survey of the Outram Park backend workspace.

Parses Cargo.toml manifests only. Executes nothing from the clones.
Usage: python -I outram_crates.py <backend_root> <out_tsv>
"""
import os
import re
import sys
from pip._vendor import tomli as tomllib
import json

root = sys.argv[1]
out = sys.argv[2]

ws = tomllib.load(open(os.path.join(root, "Cargo.toml"), "rb"))
wspkg = ws.get("workspace", {}).get("package", {})
wsdeps = ws.get("workspace", {}).get("dependencies", {})
members = ws.get("workspace", {}).get("members", [])
print("workspace.package:", json.dumps({k: wspkg.get(k) for k in wspkg}, default=str))
print("members:", len(members))
print("workspace.dependencies:", len(wsdeps))

rows = []
crates_dir = os.path.join(root, "crates")
for name in sorted(os.listdir(crates_dir)):
    d = os.path.join(crates_dir, name)
    mf = os.path.join(d, "Cargo.toml")
    if not os.path.isfile(mf):
        continue
    t = tomllib.load(open(mf, "rb"))
    p = t.get("package", {})
    lic = p.get("license")
    if isinstance(lic, dict):
        lic = "workspace:" + str(wspkg.get("license"))
    ver = p.get("version")
    if isinstance(ver, dict):
        ver = "workspace:" + str(wspkg.get("version"))
    deps = dict(t.get("dependencies", {}))
    tdeps = t.get("target", {})
    for tk, tv in tdeps.items():
        for dk, dv in tv.get("dependencies", {}).items():
            deps.setdefault(dk, dv)
    nloc = 0
    ntest = 0
    nfile = 0
    nsafe = 0
    for dp, dn, fn in os.walk(d):
        if "target" in dn:
            dn.remove("target")
        for f in fn:
            if f.endswith(".rs"):
                nfile += 1
                try:
                    s = open(os.path.join(dp, f), encoding="utf-8", errors="replace").read()
                except OSError:
                    continue
                nloc += s.count("\n")
                ntest += len(re.findall(r"#\[test\]", s))
                nsafe += len(re.findall(r"\bunsafe\b", s))
    rows.append(
        dict(
            crate=name,
            pkg=p.get("name"),
            version=ver,
            license=lic,
            desc=(p.get("description") or "")[:140] if not isinstance(p.get("description"), dict) else "",
            ndeps=len(deps),
            deps=",".join(sorted(deps)),
            rs_files=nfile,
            rs_lines=nloc,
            tests=ntest,
            unsafe_tokens=nsafe,
            features=",".join(t.get("features", {}).keys()),
            build_rs=os.path.isfile(os.path.join(d, "build.rs")),
        )
    )

with open(out, "w", encoding="utf-8") as fh:
    keys = list(rows[0].keys())
    fh.write("\t".join(keys) + "\n")
    for r in rows:
        fh.write("\t".join(str(r[k]) for k in keys) + "\n")

print("crates:", len(rows))
lic_counts = {}
for r in rows:
    lic_counts[str(r["license"])] = lic_counts.get(str(r["license"]), 0) + 1
print("license counts:", lic_counts)
print("TOTAL rs_lines", sum(r["rs_lines"] for r in rows), "tests", sum(r["tests"] for r in rows))
for r in rows:
    print(
        f'{r["crate"]:48s} {str(r["version"]):22s} {str(r["license"]):28s} '
        f'files={r["rs_files"]:5d} lines={r["rs_lines"]:8d} tests={r["tests"]:6d} '
        f'unsafe={r["unsafe_tokens"]:5d} deps={r["ndeps"]:3d} build.rs={r["build_rs"]}'
    )
