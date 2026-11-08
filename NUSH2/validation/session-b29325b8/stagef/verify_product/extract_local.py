import sys
sys.argv = [sys.argv[0], sys.argv[1], sys.argv[2]]
exec(open(sys.argv[0].replace("extract_local.py", "extract_lib.py"), encoding="utf-8").read())
office(REPO / "presentation/NUSH2_Speaker_Script.docx", "script_docx")
office(REPO / "presentation/NUSH2_QA_Prep.docx", "qa_docx")
pdf(REPO / "presentation/render/NUSH2_Speaker_Script.pdf", "script_pdf")
pdf(REPO / "presentation/render/NUSH2_QA_Prep.pdf", "qa_pdf")
with open(OUT / "hits.txt", "w", encoding="utf-8") as f:
    for label, k, n, ctx in rows:
        f.write(f"{k}\t{label}\t{n}\n")
        for c in ctx:
            f.write(f"    {c}\n")
summary = {}
for label, k, n, ctx in rows:
    summary[k] = summary.get(k, 0) + n
print(json.dumps(summary))
