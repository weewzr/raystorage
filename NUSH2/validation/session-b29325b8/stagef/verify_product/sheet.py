"""Contact sheet: python sheet.py out.png cols cell_px file1 file2 ..."""
import sys
import pymupdf

out, cols, cell = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
files = sys.argv[4:]
rows = (len(files) + cols - 1) // cols
lab = 14
doc = pymupdf.open()
page = doc.new_page(width=cols * cell, height=rows * (cell + lab))
for i, f in enumerate(files):
    r, c = divmod(i, cols)
    x0, y0 = c * cell, r * (cell + lab)
    rect = pymupdf.Rect(x0 + 2, y0 + lab, x0 + cell - 2, y0 + lab + cell - 4)
    try:
        page.insert_image(rect, filename=f, keep_proportion=True)
    except Exception as e:  # noqa: BLE001
        page.insert_text((x0 + 4, y0 + 40), f"ERR {e}", fontsize=8)
    name = f.replace("\\", "/").split("/")[-1]
    page.insert_text((x0 + 3, y0 + 11), name[:40], fontsize=9)
pix = page.get_pixmap(dpi=72)
pix.save(out)
print(out, pix.width, pix.height)
