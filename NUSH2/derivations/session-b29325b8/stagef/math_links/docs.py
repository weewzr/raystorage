import sys, re
for f in sys.argv[1:]:
    lines = open(f, encoding="utf-8").read().splitlines()
    print("=====", f)
    for i, l in enumerate(lines):
        if re.match(r"^\s*(pub(\([^)]*\))?\s+)?(fn|const|struct|enum|static)\s+\w+", l) and not re.match(r"^\s*fn\s+(eq_|[a-z_]+\(\)\s*\{)", l):
            if "#[test]" in lines[i-1]: continue
            j = i - 1
            doc = []
            while j >= 0 and (lines[j].strip().startswith("///") or lines[j].strip().startswith("#[")):
                if lines[j].strip().startswith("///"): doc.insert(0, lines[j].strip()[3:].strip())
                j -= 1
            d = " ".join(doc)
            print(f"{i+1}: {l.strip()[:110]}  || {d[:230]}")
