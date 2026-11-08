"""List value macros used in the derivation sections and which are undefined."""
import re
import sys
import pathlib
from collections import Counter

root = pathlib.Path(sys.argv[1])
defined = set()
for p in [root / 'macros.tex', root / 'generated' / 'values.tex', root / 'preamble.tex']:
    defined |= set(re.findall(r'\\(?:newcommand|renewcommand|def)\{?\\(\w+)', p.read_text(encoding='utf-8')))
# macros defined inside section files
for p in sorted((root / 'sections').glob('*.tex')):
    defined |= set(re.findall(r'\\(?:newcommand|renewcommand|def|providecommand)\{?\\(\w+)', p.read_text(encoding='utf-8')))
used = {}
for p in sorted((root / 'sections').glob('*.tex')):
    t = re.sub(r'(?<!\\)%.*', '', p.read_text(encoding='utf-8'))
    for m in re.findall(r'\\([vc][A-Z]\w*)', t):
        used.setdefault(m, set()).add(p.name.split('_')[0])
und = {k: v for k, v in used.items() if k not in defined}
print(len(defined), 'defined;', len(used), 'used v/c macros;', len(und), 'undefined')
c = Counter()
for k, v in und.items():
    for s in v:
        c[s] += 1
print(dict(c))
mode = sys.argv[2] if len(sys.argv) > 2 else 'list'
if mode == 'list':
    for sec in sorted(c):
        print(sec, ' '.join(sorted(k for k, v in und.items() if sec in v)))
