import sys
import pymupdf
sys.stdout.reconfigure(encoding="utf-8")
a, b = pymupdf.open(sys.argv[1]), pymupdf.open(sys.argv[2])
print("pages", a.page_count, b.page_count)
diff = 0
for i in range(min(a.page_count, b.page_count)):
    ta, tb = a[i].get_text().strip(), b[i].get_text().strip()
    if ta != tb:
        diff += 1
        print("text differs on page", i + 1)
    pa = a[i].get_pixmap(matrix=pymupdf.Matrix(0.3, 0.3)).samples
    pb = b[i].get_pixmap(matrix=pymupdf.Matrix(0.3, 0.3)).samples
    if pa != pb:
        n = sum(1 for x, y in zip(pa, pb) if abs(x - y) > 24)
        if n > 50:
            print("pixels differ on page", i + 1, n)
print("text-diff pages", diff)
