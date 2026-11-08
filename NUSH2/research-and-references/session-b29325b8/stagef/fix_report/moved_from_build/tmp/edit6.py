s=open('tables.py',encoding='utf-8').read()
for a,b in (('"{{GAS_CENTRAL|f0}}, high {{GAS_HIGH|f0}}"','"Central {{GAS_CENTRAL|f0}}, high {{GAS_HIGH|f0}}"'),
            ('"{{OCC_CENTRAL|f0}}, low {{OCC_LOW|f0}}"','"Central {{OCC_CENTRAL|f0}}, low {{OCC_LOW|f0}}"'),
            ('"{{TS_CENTRAL|f0}}, low {{TS_LOW|f1}}"','"Central {{TS_CENTRAL|f0}}, low {{TS_LOW|f1}}"'),
            ('widths=[34, 31, 31, 31, 32], align="lrrrl"','widths=[36, 25, 25, 25, 48], align="lrrrl"')):
    assert a in s
    s=s.replace(a,b)
open('tables.py','w',encoding='utf-8',newline='\n').write(s)
s=open('run_all.py',encoding='utf-8').read()
a='''    if "--no-qa" not in sys.argv:'''
b='''    (B / "logs" / "pdf_route.txt").write_text("P1, ExportAsFixedFormat with heading bookmarks\n", encoding="utf-8")
    if "--no-qa" not in sys.argv:'''
assert a in s
s=s.replace(a,b)
open('run_all.py','w',encoding='utf-8',newline='\n').write(s)
