def sub(p, pairs):
    s=open(p,encoding='utf-8').read()
    for a,b in pairs:
        if a not in s: print("MISSING in",p,":",a[:70])
        s=s.replace(a,b)
    open(p,'w',encoding='utf-8',newline='\n').write(s)
sub('content/04_economics.md',[('no Singapore-Indonesia tariff has been published [@metipq,navigant]','no published Singapore-Indonesia tariff was found [@metipq,navigant]')])
sub('qa_report.py',[
 ('"python-docx", "S$546", "US$80/t", "43.5 MMSCFD", "matriculation"]','"python-docx", "S$546", "US$80/t", "43.5 MMSCFD", "matriculation", "Slide 6", "the deck", "Case Q8"]'),
 ('texts = {"docx": re.sub(r"<[^>]+>", "", xml), "pdf": alltext, "core": core + app}','texts = {"docx": re.sub(r"<[^>]+>", "", xml), "pdf": alltext, "core": core + app, "pdfmeta": json.dumps(pdf.metadata)}'),
 ('" ; ".join(table_blob)))','" ; ".join(table_blob)))\n    raw_money_tables = [c for c in table_blob if "S$" in re.sub(r"\{\{[^}]+\}\}|\[\[[^\]]+\]\]", " ", c)]'),
 ('record(10, "Number lint: content numbers are tokens or literals", "PASS" if not stray and not bad_s else "FAIL", f"stray {stray[:20]}, raw money {len(bad_s)}")','record(10, "Number lint: content numbers are tokens or literals", "PASS" if not stray and not bad_s and not raw_money_tables else "FAIL", f"stray {stray[:20]}, raw money {len(bad_s)} in content, {len(raw_money_tables)} in tables")'),
])
sub('resolve_numbers.py',[
 ('SRC = ROOT / "build" / "numbers.json"','SRC = Path(os.environ.get("NUSH2_NUMBERS", ROOT / "build" / "numbers.json"))'),
 ('OUT = B / "numbers_resolved.json"','OUT = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))'),
 ('import math\nimport sys','import math\nimport os\nimport sys'),
])
sub('build_report.py',[
 ('import json\nimport re\nimport sys','import json\nimport os\nimport re\nimport sys'),
 ('OUT = B / "tmp" / "report_raw.docx"','OUT = Path(os.environ.get("NUSH2_RAW_DOCX", B / "tmp" / "report_raw.docx"))\nRESOLVED = Path(os.environ.get("NUSH2_RESOLVED", B / "numbers_resolved.json"))'),
 ('res = json.loads((B / "numbers_resolved.json").read_text(encoding="utf-8"))','res = json.loads(RESOLVED.read_text(encoding="utf-8"))'),
])
