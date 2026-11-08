import pymupdf as fitz, sys, collections
d = fitz.open(sys.argv[1])
cnt = collections.Counter(); ex = {}
for i, p in enumerate(d):
    for b in p.get_text("dict")["blocks"]:
        for l in b.get("lines", []):
            for s in l["spans"]:
                t = s["text"].strip()
                if not t: continue
                k = (round(s["size"],1), s["font"])
                cnt[k] += len(t)
                ex.setdefault(k, []).append((i+1, t[:50]))
for k, v in sorted(cnt.items(), key=lambda kv: -kv[1]):
    print(k, v, ex[k][:3])
