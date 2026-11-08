import pymupdf, re, sys, os
sys.stdout.reconfigure(encoding="utf-8")
SP = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
p = SP + "/findings/dl_econ/cho/cho_sd.pdf"
print("cho size", os.path.getsize(p), open(p, "rb").read(200))
try:
    d = pymupdf.open(p)
    print("cho pages", d.page_count)
except Exception as e:
    print("cho not a PDF:", e)
d = pymupdf.open(SP + "/findings/dl_econ/eria/ch15_2023.pdf")
print("eria pages", d.page_count)
for i, pg in enumerate(d):
    t = re.sub(r"\s+", " ", pg.get_text())
    for m in re.finditer(r"(90 ?%|90 percent|hydrogen|CCUS|carbon capture)", t, re.I):
        s = max(0, m.start() - 130)
        print(f"[pdf p{i+1}] ...{t[s:m.end()+130]}...")
