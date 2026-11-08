import json
from pathlib import Path
H = Path(__file__).parent / "inv"
rows = json.loads((H / "inventory.json").read_text(encoding="utf8"))
print("== root and research-root PDFs not in numbered folders ==")
for r in sorted(rows, key=lambda x: x["rel"].lower()):
    p = r["rel"]
    if r["ext"] == ".pdf" and p.startswith("HTGR_SMR_MEA_CCS_Research_Local\\") and p.count("\\") == 1:
        print(p.split("\\", 1)[1], "| pp", r.get("pages"), "| doi", r.get("doi_guess"), "| T:", (r.get("meta_title") or "")[:70], "| ", (r.get("first_text") or "")[:150])
print("\n== root-level pdfs unique to root ==")
d = json.loads((H / "duplicates.json").read_text(encoding="utf8"))
dupset = {x for v in d.values() for x in v}
for r in rows:
    p = r["rel"]
    if r["ext"] == ".pdf" and "\\" not in p and p not in dupset:
        print(p, "| pp", r.get("pages"), "| ", (r.get("first_text") or "")[:120])
print("\n== Lectures and attachment pdfs ==")
for r in rows:
    p = r["rel"]
    if r["ext"] in (".pdf",".PDF") and ("attachments" in p or "literature" in p.lower() or "external" in p):
        print(p.split("\\")[-1], "| pp", r.get("pages"), "| ", (r.get("first_text") or "")[:100])
