"""Read PDF page count through HTTP Range requests held in memory. Saves nothing."""
import io, sys, urllib.request
from pypdf import PdfReader
sys.stdout.reconfigure(encoding="utf-8")

class RF(io.RawIOBase):
    B = 1 << 17
    def __init__(self, url):
        self.url, self.pos, self.cache, self.got = url, 0, {}, 0
        req = urllib.request.Request(url, headers={"Range": "bytes=0-0"})
        with urllib.request.urlopen(req, timeout=60) as r:
            self.size = int(r.headers["Content-Range"].split("/")[1])
    def readable(self): return True
    def seekable(self): return True
    def tell(self): return self.pos
    def seek(self, o, w=0):
        self.pos = o if w == 0 else (self.pos + o if w == 1 else self.size + o)
        return self.pos
    def blk(self, i):
        if i not in self.cache:
            s = i * self.B; e = min(self.size - 1, s + self.B - 1)
            req = urllib.request.Request(self.url, headers={"Range": f"bytes={s}-{e}"})
            with urllib.request.urlopen(req, timeout=60) as r:
                self.cache[i] = r.read()
            self.got += len(self.cache[i])
        return self.cache[i]
    def read(self, n=-1):
        if n < 0: n = self.size - self.pos
        n = min(n, self.size - self.pos)
        out = bytearray()
        while n > 0:
            i = self.pos // self.B; off = self.pos % self.B
            d = self.blk(i)[off:off + n]
            if not d: break
            out += d; self.pos += len(d); n -= len(d)
        return bytes(out)

if __name__ == "__main__":
    u = sys.argv[1]
    f = RF(u)
    r = PdfReader(f)
    n = len(r.pages)
    print("size", f.size, "pages", n, "bytes_read_in_memory", f.got)
    if len(sys.argv) > 2:
        for p in sys.argv[2:]:
            p = int(p)
            t = (r.pages[p - 1].extract_text() or "").strip().replace("\n", " ")
            print(f"--- PDF p{p}:", t[:300])
    print("bytes_read_in_memory", f.got)
