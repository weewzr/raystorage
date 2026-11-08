"""Scan every XML part of Office files for withdrawn figures and SMR usage; dump notes text."""
import re, sys, zipfile, html
sys.stdout.reconfigure(encoding='utf-8')
pats = {
    'S$546': r'546', 'US$80/t': r'US\$\s?80\s?/\s?t', '106.5': r'106\.5', '99.4': r'99\.4',
    '7.2': r'(?<![\d.])7\.2(?![\d])', '43.5': r'43\.5', 'S$96': r'S\$\s?96(?![\d.])',
    'bareSMR': r'\bSMRs?\b', 'emdash': '—',
}
for path in sys.argv[1:]:
    z = zipfile.ZipFile(path)
    print('==', path)
    for n in z.namelist():
        if not n.endswith(('.xml', '.rels')):
            continue
        raw = z.read(n).decode('utf-8', 'replace')
        text = html.unescape(re.sub(r'<[^>]+>', ' ', raw))
        text = re.sub(r'\s+', ' ', text)
        for k, p in pats.items():
            for m in re.finditer(p, text):
                ctx = text[max(0, m.start() - 70): m.end() + 70]
                print(f'  {n}: {k}: ...{ctx}...')
