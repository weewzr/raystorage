import subprocess, os
from mutate import REPO, SCR, env
rel = "engine/plant/src/export.rs"
old = "num(p.private_capital),"; new = "num(p.grant),"
path = f"{REPO}/{rel}"
src = open(path, encoding="utf-8", newline="").read()
assert src.count(old) == 1
out = f"{SCR}/export_r5"; os.makedirs(out, exist_ok=True)
open(path, "w", encoding="utf-8", newline="").write(src.replace(old, new))
try:
    r = subprocess.run(["cargo", "run", "--release", "-p", "nush2_cli", "--", "export", "--out", out], cwd=REPO, env=env, capture_output=True, text=True)
    print("export exit", r.returncode)
    c = subprocess.run(["python", "scripts/compare_results.py", "results", out], cwd=REPO, capture_output=True, text=True)
    print("compare exit", c.returncode)
    print("\n".join((c.stdout + c.stderr).splitlines()[-8:]))
finally:
    open(path, "w", encoding="utf-8", newline="").write(src)
    subprocess.run(["git", "checkout", "--", rel], cwd=REPO)
