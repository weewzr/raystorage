import pymupdf, re
p = r"C:\Users\Alf\.claude\projects\C--Ray-Claude-Ray-Debug-VS-Code\b29325b8-d1b9-49cd-bfe6-262261d7ffef\tool-results\webfetch-1793578732848-2x1b3w.pdf"
d = pymupdf.open(p)
print(d.page_count, d.metadata.get("title"))
pat = re.compile(r"(?i)decommission|licens|pre-?construction|owner|scal(e|ing) exponent|economies of scale|kWt|HTGR|Table 1\.")
for i in range(d.page_count):
    t = d[i].get_text()
    hits = [m.group(0).lower() for m in pat.finditer(t)]
    if hits:
        print(i + 1, sorted(set(hits))[:9], len(hits))
