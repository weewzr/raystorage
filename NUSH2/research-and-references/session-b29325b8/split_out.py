import json, re
from pathlib import Path
src = Path(r"C:\Users\Alf\AppData\Local\Temp\claude\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tasks\wwgt0oa2w.output")
raw = src.read_text(encoding="utf8")
print("len", len(raw), "head", raw[:120])
try:
    d = json.loads(raw)
except Exception as e:
    print("direct json fail", e)
    i = raw.find('{"findings"')
    d, _ = json.JSONDecoder().raw_decode(raw[i:])
print("keys", list(d.keys()))
out = Path("wf"); out.mkdir(exist_ok=True)
for k, v in d.items():
    if k == "findings":
        for f in v:
            (out / f"f_{f['key']}.json").write_text(json.dumps(f, indent=1, ensure_ascii=False), encoding="utf8")
            print(f["key"], len(json.dumps(f)), "facts", len(f.get("key_facts", [])), "conflicts", len(f.get("conflicts", [])), "decisions", len(f.get("decisions", [])), "gaps", len(f.get("gaps", [])))
    else:
        (out / f"{k}.json").write_text(json.dumps(v, indent=1, ensure_ascii=False), encoding="utf8")
        print(k, len(json.dumps(v)))
