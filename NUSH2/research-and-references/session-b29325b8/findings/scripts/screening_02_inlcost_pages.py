import pymupdf, re, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
p = r"C:\Users\Alf\.claude\projects\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tool-results\webfetch-1793578732848-2x1b3w.pdf"
d = pymupdf.open(p)
for i in (22, 25, 36, 47):
    print("=== INL 72972 PDF p", i)
    print(d[i - 1].get_text()[:3800])
for i in (19, 33, 34, 40, 67):
    t = d[i - 1].get_text()
    for m in re.finditer(r"(?i)licens|decommission", t):
        s = max(0, m.start() - 200)
        print("--- PDF p", i, t[s:m.end() + 240].replace("\n", " "))
p2 = r"C:\Users\Alf\.claude\projects\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tool-results\webfetch-1793578754628-8o66hy.pdf"
e = pymupdf.open(p2)
print("NZW pages", e.page_count, e.metadata.get("title"))
pat = re.compile(r"(?i)decommission")
n = 0
for i in range(e.page_count):
    t = e[i].get_text()
    m = pat.search(t)
    if m and n < 12:
        n += 1
        s = max(0, m.start() - 260)
        print("--- NZW PDF p", i + 1, t[s:m.end() + 420].replace("\n", " "))
