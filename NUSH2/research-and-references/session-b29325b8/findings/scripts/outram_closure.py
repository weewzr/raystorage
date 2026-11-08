"""Static dependency closure and wasm hazard scan. Executes nothing from the clone.

For each candidate crate, walk normal [dependencies] of workspace members,
keeping tables that apply on wasm32 (plain, or target tables that do not
exclude wasm32), then extend through Cargo.lock for external packages.

The Cargo.lock extension over-approximates: it lists every dependency edge
that any workspace feature set activates, so the external counts are upper
bounds, not exact wasm build graphs.

Usage: python -I outram_closure.py <backend_root> <out_md>
"""
import os
import re
import sys
from collections import defaultdict
from pip._vendor import tomli

root = sys.argv[1]
out_md = sys.argv[2]
crates_dir = os.path.join(root, "crates")

CANDIDATES = [
    "petir",
    "tampines-steam-tables",
    "outram-park-fork-coolprop",
    "outram-park-fork-dwsim-libs",
    "dover",
    "tuas_boussinesq_solver",
    "chem-eng-real-time-process-control-simulator",
    "outram-foam-basic-lib",
    "tampines",
]

ws = tomli.load(open(os.path.join(root, "Cargo.toml"), "rb"))
ws_deps = ws["workspace"]["dependencies"]

# package name -> crate dir
pkg_to_dir = {}
manifests = {}
for d in sorted(os.listdir(crates_dir)):
    mf = os.path.join(crates_dir, d, "Cargo.toml")
    if os.path.isfile(mf):
        t = tomli.load(open(mf, "rb"))
        pkg_to_dir[t["package"]["name"]] = d
        manifests[d] = t


def resolve_name(key, spec):
    if isinstance(spec, dict):
        if "package" in spec:
            return spec["package"]
        if spec.get("workspace"):
            w = ws_deps.get(key, {})
            if isinstance(w, dict) and "package" in w:
                return w["package"]
    return key


def applies_on_wasm(cfg):
    c = cfg.replace(" ", "")
    if 'not(target_arch="wasm32")' in c:
        return False
    if 'target_os="android"' in c and "not(" not in c:
        return False
    return True


def direct_deps(d):
    """(name, optional) for normal deps applying on wasm32."""
    t = manifests[d]
    out = []
    for k, v in t.get("dependencies", {}).items():
        opt = isinstance(v, dict) and v.get("optional", False)
        out.append((resolve_name(k, v), opt))
    for cfg, tv in t.get("target", {}).items():
        if applies_on_wasm(cfg):
            for k, v in tv.get("dependencies", {}).items():
                opt = isinstance(v, dict) and v.get("optional", False)
                out.append((resolve_name(k, v), opt))
    return out


lock = tomli.load(open(os.path.join(root, "Cargo.lock"), "rb"))
lock_deps = defaultdict(set)
lock_ver = {}
for p in lock["package"]:
    n = p["name"]
    lock_ver.setdefault(n, []).append(p["version"])
    for dep in p.get("dependencies", []):
        lock_deps[n].add(dep.split(" ")[0])

HAZARD_EXT = [
    "rayon",
    "getrandom",
    "rand",
    "wgpu",
    "tokio",
    "faer",
    "peroxide",
    "openblas-src",
    "ndarray-linalg",
    "zip",
    "uuid",
    "ring",
    "wasm-bindgen",
    "js-sys",
    "web-sys",
]

lines = []
lines.append("| crate | workspace crates reached | external pkgs (lock, upper bound) | hazards present in closure |")
lines.append("|---|---|---|---|")
closures = {}
for c in CANDIDATES:
    seen_ws = []
    seen_ext = set()
    stack = [(c, False)]
    visited = set()
    while stack:
        d, _ = stack.pop()
        if d in visited:
            continue
        visited.add(d)
        seen_ws.append(d)
        for name, opt in direct_deps(d):
            if opt:
                continue
            if name in pkg_to_dir:
                stack.append((pkg_to_dir[name], False))
            else:
                seen_ext.add(name)
    ext_all = set()
    st = list(seen_ext)
    while st:
        n = st.pop()
        if n in ext_all:
            continue
        ext_all.add(n)
        for k in lock_deps.get(n, ()):
            if k not in ext_all and k not in pkg_to_dir:
                st.append(k)
    haz = sorted(h for h in HAZARD_EXT if h in ext_all)
    closures[c] = (sorted(seen_ws), sorted(seen_ext), ext_all)
    lines.append(f"| {c} | {len(seen_ws)}: {', '.join(sorted(seen_ws))} | {len(ext_all)} | {', '.join(haz) if haz else 'none'} |")

# Runtime hazards inside non-test library source.
pat = {
    "thread::spawn": re.compile(r"\bthread::spawn\b"),
    "Instant::now": re.compile(r"\bInstant::now\b"),
    "SystemTime": re.compile(r"\bSystemTime\b"),
    "std::fs": re.compile(r"\bstd::fs\b|\bfs::(read|write|File|create)"),
    "rayon": re.compile(r"\brayon\b|\.par_iter\b"),
}
lines2 = [
    "",
    "| crate | lib .rs files scanned | thread::spawn | Instant::now | SystemTime | std::fs | rayon |",
    "|---|---|---|---|---|---|---|",
]
for c in CANDIDATES:
    src = os.path.join(crates_dir, c, "src")
    cnt = {k: 0 for k in pat}
    n = 0
    for dp, dn, fn in os.walk(src):
        rel = os.path.relpath(dp, src).replace("\\", "/")
        if "/tests" in "/" + rel or rel.endswith("tests") or "/bin" in "/" + rel:
            continue
        for f in fn:
            if not f.endswith(".rs") or f == "tests.rs" or f.endswith("_tests.rs"):
                continue
            n += 1
            s = open(os.path.join(dp, f), encoding="utf-8", errors="replace").read()
            # cut any trailing #[cfg(test)] module
            i = s.find("#[cfg(test)]")
            if i >= 0:
                s = s[:i]
            for k, rx in pat.items():
                cnt[k] += len(rx.findall(s))
    lines2.append(
        f"| {c} | {n} | {cnt['thread::spawn']} | {cnt['Instant::now']} | {cnt['SystemTime']} | {cnt['std::fs']} | {cnt['rayon']} |"
    )

text = "\n".join(lines + lines2) + "\n"
open(out_md, "w", encoding="utf-8").write(text)
print(text)
for c, (w, e, a) in closures.items():
    print(c, "direct external:", e)
print("wasm-bindgen versions in lock:", lock_ver.get("wasm-bindgen"))
