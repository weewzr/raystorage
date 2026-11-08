import zipfile, re, sys
z = zipfile.ZipFile(sys.argv[1])
for n in sorted(z.namelist()):
    if n.startswith('ppt/notesSlides/notesSlide') or n.startswith('ppt/slides/slide'):
        if not n.endswith('.xml'): continue
        t = z.read(n).decode('utf8','ignore')
        txt = ' '.join(re.findall(r'<a:t>([^<]*)</a:t>', t))
        if re.search(r'carbon tax|\bETS\b|per tonne|S\$ ?\d+ ?/ ?t|2030|2035', txt, re.I):
            for m in re.finditer(r'.{0,200}carbon tax.{0,300}', txt, re.I):
                print(n, '::', m.group(0)); print()
