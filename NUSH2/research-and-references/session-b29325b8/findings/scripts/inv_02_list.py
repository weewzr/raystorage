import json, os, csv, collections, sys
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
inv = json.load(open(SP + "/inv/inventory.json", encoding="utf-8"))
dup = json.load(open(SP + "/inv/duplicates.json", encoding="utf-8"))
REPO = "Implementation-of-Nuclear-Assisted-SMR-with-Carbon-Capture-in-Singapore-main"
print("=== NON-REPO FILES ===")
for e in sorted(inv, key=lambda x: x["rel"].lower()):
    if e["rel"].startswith(REPO):
        continue
    print(e["rel"], "|", e["bytes"], "|", (e.get("sha256") or "")[:10], "| p", e.get("pages", ""))
print("=== REPO: non md/rs etc ===")
for e in sorted(inv, key=lambda x: x["rel"].lower()):
    if e["rel"].startswith(REPO) and e["ext"] in (".pdf", ".docx", ".xlsx", ".xls", ".xlsm", ".pptx", ".PDF", ".csv", ".png", ".jpg", ".bib", ".tex"):
        print(e["rel"][len(REPO)+1:], "|", e["bytes"], "|", (e.get("sha256") or "")[:10], "| p", e.get("pages", ""))
print("=== DUPS ===")
for k, v in dup.items():
    print(k[:10], len(v))
    for x in v:
        print("   ", x)
