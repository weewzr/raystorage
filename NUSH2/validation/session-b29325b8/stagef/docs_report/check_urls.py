#!/usr/bin/env python3
"""Plan step 4. GET every reference URL and log the outcome. Nothing is saved except the status."""
import csv
import ssl
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

B = Path(__file__).resolve().parent
UA = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36"
CTX = ssl.create_default_context()


def probe(row):
    url = row["url"]
    if not url:
        return row["key"], "", "no url"
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Range": "bytes=0-2047"})
    try:
        with urllib.request.urlopen(req, timeout=25, context=CTX) as r:
            r.read(2048)
            return row["key"], url, str(r.status)
    except urllib.error.HTTPError as e:
        return row["key"], url, str(e.code)
    except Exception as e:
        return row["key"], url, type(e).__name__


def main():
    rows = list(csv.DictReader(open(B / "refs.csv", encoding="utf-8")))
    with ThreadPoolExecutor(8) as ex:
        out = list(ex.map(probe, rows))
    with open(B / "logs" / "url_status.csv", "w", encoding="utf-8", newline="") as f:
        w = csv.writer(f)
        w.writerow(["key", "url", "status"])
        w.writerows(out)
    bad = [x for x in out if x[2] not in ("200", "206", "no url")]
    print(f"{len(out)} refs, {len(bad)} not 200")
    for k, u, s in bad:
        print(k, s, u[:90])


if __name__ == "__main__":
    main()
