"""Dump text of selected PDF pages to a UTF-8 file. Usage: dump.py pdf out pages(1-based, comma or a-b) [mode]"""
import sys
import fitz


def pages(spec, n):
    out = []
    for part in spec.split(","):
        if "-" in part:
            a, b = part.split("-")
            out.extend(range(int(a), int(b) + 1))
        elif part == "all":
            out.extend(range(1, n + 1))
        else:
            out.append(int(part))
    return [p for p in out if 1 <= p <= n]


def main():
    pdf, out, spec = sys.argv[1], sys.argv[2], sys.argv[3]
    mode = sys.argv[4] if len(sys.argv) > 4 else "text"
    d = fitz.open(pdf)
    with open(out, "w", encoding="utf-8") as f:
        f.write(f"{pdf} pages={d.page_count}\n")
        for p in pages(spec, d.page_count):
            f.write(f"\n=== PDF PAGE {p}\n")
            if mode == "blocks":
                for b in d[p - 1].get_text("blocks"):
                    f.write(b[4].replace("\n", " ") + "\n")
            else:
                f.write(d[p - 1].get_text())


main()
