import sys, zipfile, re
sys.stdout.reconfigure(encoding="utf-8")
pat = re.compile(sys.argv[2], re.I)
for f in sys.argv[1].split(","):
    z = zipfile.ZipFile(f); hits = 0
    for n in z.namelist():
        if n.endswith((".xml", ".rels")):
            t = z.read(n).decode("utf8", "replace")
            for m in pat.finditer(t):
                hits += 1
                if hits <= 6: print(f.split("/")[-1], n, "...", t[max(0, m.start()-60):m.end()+40].replace("\n", " "))
    print(f.split("/")[-1], "hits:", hits)
