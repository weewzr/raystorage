import re, sys, zipfile, io
from pathlib import Path
import pymupdf

RX = re.compile(r"(?<![A-Za-z])[A-Za-z]:[\\/](?![\\/])|\\\\[A-Za-z0-9_.-]+\\|/Users/|\\Users\\|/home/|AppData|scratchpad"
                r"|\bAlf\b|OneDrive|file:///|\bY3S1\b|Ray Claude|weewzrresearch|@gmail|@u\.nus|@nus\.edu")
assert RX.search("C:/Users/x") and RX.search("/Users/a") and RX.search(r"C:\Users\x") and RX.search("a\\Users\\b")
assert not RX.search("https://example.org/a")


def scan(label, text):
    for m in RX.finditer(text):
        print(label, "|", " ".join(text[max(0, m.start() - 50):m.end() + 50].split()))


for f in sys.argv[2:]:
    p = Path(sys.argv[1]) / f
    if f.endswith((".docx", ".pptx")):
        z = zipfile.ZipFile(p)
        for i in z.infolist():
            raw = z.read(i)
            low = i.filename.lower()
            if low.endswith(".xlsx"):
                zz = zipfile.ZipFile(io.BytesIO(raw))
                for j in zz.infolist():
                    scan(f + "!" + i.filename + "!" + j.filename, zz.read(j).decode("utf-8", "replace"))
            elif low.endswith(".bin"):
                scan(f + "!" + i.filename, raw.decode("latin-1"))
                scan(f + "!" + i.filename + " u16", raw.decode("utf-16le", "replace"))
            elif low.endswith((".png", ".jpg", ".jpeg", ".gif", ".emf", ".wmf")):
                # text chunks only
                for chunk in re.findall(rb"[\x20-\x7e]{6,}", raw):
                    scan(f + "!" + i.filename + " ascii", chunk.decode())
            else:
                scan(f + "!" + i.filename, raw.decode("utf-8", "replace"))
    elif f.endswith(".pdf"):
        d = pymupdf.open(p)
        scan(f + " meta", str(d.metadata) + (d.get_xml_metadata() or ""))
        for pg in d:
            scan(f"{f} p{pg.number + 1}", pg.get_text())
            for l in pg.get_links():
                scan(f"{f} p{pg.number + 1} link", str(l.get("uri")) + str(l.get("file")))
        for i in range(d.embfile_count()):
            scan(f"{f} att{i}", d.embfile_get(i).decode("utf-8", "replace"))
        for x in range(1, d.xref_length()):
            try:
                scan(f"{f} xref{x}", d.xref_object(x))
            except Exception:  # noqa: BLE001
                pass
    else:
        scan(f, p.read_bytes().decode("utf-8", "replace"))
print("done")
