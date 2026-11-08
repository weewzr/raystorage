import json, importlib.util, time
from pathlib import Path
H = Path(__file__).parent / "inv"
rows = json.loads((H / "inventory.json").read_text(encoding="utf8"))
print("== key types ==")
for r in sorted(rows, key=lambda x: -x["mtime"]):
    if r["ext"] in (".pptx", ".mp4", ".xlsx", ".xlsm", ".xls", ".bib", ".toml", ".txt", ".lnk", ".download", ".yml"):
        print(time.strftime("%Y-%m-%d %H:%M", time.localtime(r["mtime"])), round(r["bytes"] / 1e6, 2), r["rel"])
print("\n== all docx, newest first ==")
for r in sorted(rows, key=lambda x: -x["mtime"]):
    if r["ext"] == ".docx" and "Midterm_Reflection" not in r["rel"]:
        print(time.strftime("%Y-%m-%d %H:%M", time.localtime(r["mtime"])), r.get("word_count"), r.get("images"), r["rel"], "|", r.get("creator"), "|", r.get("lastModifiedBy"))
print("\n== md outside the -main repo ==")
for r in sorted(rows, key=lambda x: x["rel"]):
    if r["ext"] in (".md", ".csv", ".txt") and not r["rel"].startswith("Implementation-of"):
        print(round(r["bytes"] / 1e3, 1), r["rel"])
print("\n== low text pdfs ==")
print(json.loads((H / "scanned_pdfs.json").read_text(encoding="utf8")))
print("\n== duplicate groups ==")
for k, v in json.loads((H / "duplicates.json").read_text(encoding="utf8")).items():
    print(" | ".join(v))
print("\n== modules ==")
for m in ["docx", "pptx", "openpyxl", "fitz", "pdfplumber", "cv2", "PIL", "lxml", "pytesseract", "xlrd"]:
    print(m, "ok" if importlib.util.find_spec(m) else "MISSING")
