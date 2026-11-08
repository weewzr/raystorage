from pathlib import Path


def rep(path, a, b, n=1):
    p = Path(path)
    s = p.read_bytes().decode("utf-8")
    crlf = "\r\n" in s
    s = s.replace("\r\n", "\n")
    assert s.count(a) == n, (path, a[:80], s.count(a))
    s = s.replace(a, b)
    if crlf:
        s = s.replace("\n", "\r\n")
    p.write_bytes(s.encode("utf-8"))
