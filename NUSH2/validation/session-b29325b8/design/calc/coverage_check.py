"""List numeric tokens in the deck text and Latest report (body, before references) that no old_value row mentions."""
import csv, re
from pathlib import Path
S = Path(__file__).resolve().parent.parent.parent
rows = list(csv.DictReader(open(S / "design/01_value_changes.csv", encoding="utf-8-sig")))
olds = " ".join(r["old_value"] for r in rows)
num = re.compile(r"\d[\d,]*\.?\d*")
oldnums = {n.rstrip(".,").replace(",", "") for n in num.findall(olds)}
deck = (S / "txt/deck.txt").read_text(encoding="utf-8")
# keep live slides only, drop references and template slides
parts = re.split(r"=== SLIDE (\d+) ", deck)
live = ""
for i in range(1, len(parts), 2):
    n = int(parts[i])
    if n in (11, 30, 31, 32, 33, 34, 35, 36, 37):
        continue
    t = "\n".join(l for l in parts[i + 1].splitlines() if not l.startswith("[media") and not l.startswith("[pics"))
    t = t.replace("1.7.2013", "")
    live += f"\n#S{n}\n" + t
rep = (S / "txt/report_latest.txt").read_text(encoding="utf-8")
rep = rep.split("[normal] 6 References")[0]
for name, text in (("deck", live), ("report", rep)):
    seen = {}
    for m in num.finditer(text):
        tok = m.group().rstrip(".,").replace(",", "")
        if tok not in oldnums:
            ctx = text[max(0, m.start() - 45):m.end() + 35].replace("\n", " ")
            seen.setdefault(tok, ctx)
    print("====", name, len(seen))
    for k, v in sorted(seen.items(), key=lambda kv: (len(kv[0]), kv[0])):
        print(f"{k:>10} | {v}")
