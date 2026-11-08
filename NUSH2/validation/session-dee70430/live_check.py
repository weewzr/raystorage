"""Check the live Pages site against the deployed artifact of a site run.

usage: live_check.py <run_id> <expect_commit> <work_dir> <repo_checkout>
Downloads the github-pages artifact of <run_id>, compares every file with the
live URL byte for byte, then runs scripts/selftest.py on the artifact folder.
"""
import hashlib
import json
import pathlib
import subprocess
import sys
import tarfile
import time
import urllib.request

BASE = "https://weewzr.github.io/NUSH2/"
run_id, sha, work, repo = sys.argv[1:5]
work = pathlib.Path(work)
dl = work / "dl"
site = work / "site"
dl.mkdir(parents=True, exist_ok=False)
site.mkdir(parents=True, exist_ok=False)

subprocess.run(["gh", "run", "download", run_id, "--repo", "weewzr/NUSH2",
                "-n", "github-pages", "-D", str(dl)], check=True)
tars = list(dl.glob("*.tar"))
assert len(tars) == 1, tars
with tarfile.open(tars[0]) as t:
    for m in t.getmembers():
        target = (site / m.name).resolve()
        assert str(target).startswith(str(site.resolve())), m.name
        assert not (m.issym() or m.islnk() or m.isdev()), m.name
    t.extractall(site)

files = sorted(p for p in site.rglob("*") if p.is_file())
build = json.loads((site / "build.json").read_text(encoding="utf-8"))
print("artifact files:", len(files), "build.json commit:", build.get("commit"))

mismatch = []
for p in files:
    rel = p.relative_to(site).as_posix()
    url = BASE + rel + "?nocache=" + str(int(time.time()))
    try:
        with urllib.request.urlopen(url, timeout=30) as r:
            live = r.read()
    except Exception as e:  # noqa: BLE001
        mismatch.append((rel, "fetch failed: " + str(e)))
        continue
    if live != p.read_bytes():
        mismatch.append((rel, "differs: live %s artifact %s" % (
            hashlib.sha256(live).hexdigest()[:12],
            hashlib.sha256(p.read_bytes()).hexdigest()[:12])))
print("live matches artifact:", len(files) - len(mismatch), "of", len(files))
for m in mismatch:
    print("  MISMATCH", m)

py = sys.executable
runs = [
    ["--width", "1440", "--height", "900"],
    ["--width", "375", "--height", "812", "--mobile"],
    ["--width", "1440", "--height", "900", "--reduced-motion"],
    ["--width", "375", "--height", "812", "--mobile", "--reduced-motion"],
]
codes = []
for extra in runs:
    cmd = [py, str(pathlib.Path(repo) / "scripts" / "selftest.py"), "--site", str(site),
           "--expect-commit", sha] + extra
    r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8", errors="replace")
    lines = (r.stdout + r.stderr).strip().splitlines()
    summary = [l for l in lines if any(k in l for k in ("PASS", "FAIL", "SKIP", "error", "request"))]
    print("selftest", " ".join(extra), "exit", r.returncode)
    for l in (summary[-6:] if r.returncode == 0 else summary[-25:]):
        print("   ", l[:200])
    codes.append(r.returncode)
print("RESULT:", "OK" if not mismatch and not any(codes) else "PROBLEM")
