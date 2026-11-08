"""Tighten references in findings/inl.md. A-numbers restart per document, so TEV-953 ones are named."""
p = r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad/findings/inl.md"
s = open(p, encoding="utf-8").read()
rep = [
 ("Page references give the document page and the PDF page, for example \"doc p16 (PDF p17)\". TEV-953 and TEV-961 both run one PDF page ahead of the document page. Appendix pages are cited as A-n with their PDF page.",
  "Page references give the document page and the PDF page, for example \"doc p16 (PDF p17)\". TEV-953 and TEV-961 both run one PDF page ahead of the document page. Appendix pages are cited as A-n with their PDF page. A-numbers restart in each document, so an A-number with no document named is TEV-961, and TEV-953 ones are always named."),
 ("2. **Most of the 5.7 MWe is probably recycle compression, not a side load.** The 3.9 MW",
  "2. **Most of the 5.7 MWe is probably recycle compression.** The 3.9 MW"),
 ("TEV-953 Table 2 doc p29 (PDF p30); A-5 (PDF p39) | 44.55 (84.86%) | 7.95 (15.14%, A-7 PDF p41)",
  "TEV-953 Table 2 doc p29 (PDF p30); TEV-953 A-5 (PDF p39) | 44.55 (84.86%) | 7.95 (15.14%, TEV-953 A-7, PDF p41)"),
 ("Table 2; A-16 (PDF p50); TEV-961 Case 1 | 44.76 (90.98%) | 4.44 (9.02%, A-18 PDF p52)",
  "TEV-953 Table 2; TEV-953 A-16 (PDF p50); TEV-961 Case 1 | 44.76 (90.98%) | 4.44 (9.02%, TEV-953 A-18, PDF p52)"),
 ("| Table 2; A-40 (PDF p74) | 43.5 | 0 | 91.4% of PSA reject fired in Stage 2, 8.6% recycled [CALC from A-45] |",
  "| TEV-953 Table 2; TEV-953 A-40 (PDF p74) | 43.5 | 0 | 91.4% of PSA reject fired in Stage 2, 8.6% recycled [CALC from TEV-953 A-45, PDF p79] |"),
 ("| 44.5 | HTGR 750 C ROT, **no capture** | Table 2 | 44.5 | 0, doc p22 |", "| 44.5 | HTGR 750 C ROT, **no capture** | TEV-953 Table 2 | 44.5 | 0, TEV-953 doc p22 |"),
 ("PSA reject with 56.5 mol% CO2, doc p18 |", "PSA reject with 56.5 mol% CO2, TEV-953 doc p18 |"),
 ("PSA reject with 1.3 mol% CO2, doc p20 |", "PSA reject with 1.3 mol% CO2, TEV-953 doc p20 |"),
 ("| TEV-961 **Case 6**, 925 C ROT (900 C helium) | Table 2 doc p16; A-1; A-2 (PDF p26) | 34.0 (A-18 PFD, PDF p42) | 0 | Only the N2-rich cut, \"Combust & Generate Steam\", A-16 (PDF p40) |",
  "| TEV-961 **Case 6**, 925 C ROT (900 C helium) | TEV-961 Table 2 doc p16; A-1; A-2 (PDF p26) | 34.0 (A-18 PFD, PDF p42) | 0 | Only the N2-rich cut, \"Combust & Generate Steam\", A-16 (PDF p40) |"),
 ("| Carbon lost with N2-rich cut | n/a | about 260 lbmol C/h, or 6.8% of NG carbon |", "| Carbon lost with N2-rich cut | n/a | about 260 lbmol C/h, or 6.6 to 6.8% of NG carbon |"),
 ("TEV-953 doc p20; A-43 (PDF p77); A-53 (PDF p77). Modelled as a separation block", "TEV-953 doc p20; TEV-953 A-43 (PDF p77); TEV-961 A-53 (PDF p77). Modelled as a separation block"),
 ("| TEV-953 Table 2; A-6 (PDF p40) |", "| TEV-953 Table 2; TEV-953 A-6 (PDF p40) |"),
 ("| 2,143 captured, 855 emitted, 71.5% [CALC] | Table 2 |", "| 2,143 captured, 855 emitted, 71.5% [CALC] | TEV-953 Table 2 |"),
 ("Final product composition is 99.9 mol% H2, 0.1 mol% CH4 (A-40, A-50).", "Final product composition is 99.9 mol% H2, 0.1 mol% CH4 (TEV-953 A-40; TEV-961 A-50)."),
 ("Carbon closure below uses feed carbon of 1.05379 mol C per mol NG (A-5 composition)", "Carbon closure below uses feed carbon of 1.05379 mol C per mol NG (TEV-953 A-5 composition)"),
 ("specified inputs in every case with a calculator sheet (TEV-953 doc p18, doc p20, A-53).", "specified inputs in every case with a calculator sheet (TEV-953 doc p18 and doc p20; TEV-961 A-53)."),
 ("| not local; about 17.5 to 19% [CALC] |", "| not local; about 17 to 19%, rough [CALC] |"),
 ("The same model gives 64 against the published 65.7 for WGS-EX3 |", "The same model gives about 66 against the published 65.7 for WGS-EX3 |"),
 ("All values [PUB] from A-25, A-35, A-45, A-55 (PDF pp49, 59, 69, 79), except the calculated columns.", "All values [PUB] from TEV-961 A-25, A-35, A-45, A-55 (PDF pp49, 59, 69, 79), except the calculated columns."),
]
miss = 0
for a, b in rep:
    if a not in s:
        print("MISSING:", a[:70]); miss += 1
    s = s.replace(a, b)
open(p, "w", encoding="utf-8").write(s)
print("done, missing", miss, "; em dashes:", s.count("\u2014"), "; en dashes:", s.count("\u2013"), "; chars", len(s))
