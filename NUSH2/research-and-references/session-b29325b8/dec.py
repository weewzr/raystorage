import json
from pathlib import Path
print(Path("wf/missing.json").read_text(encoding="utf8")[:400])
for f in sorted(Path("wf").glob("f_*.json")):
    d = json.loads(f.read_text(encoding="utf8"))
    print("\n=====", d["key"], "=====")
    print("SUMMARY:", d["summary"])
    print("-- DECISIONS")
    for x in d.get("decisions", []):
        print(" *", json.dumps(x, ensure_ascii=False))
