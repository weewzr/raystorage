import re
def sub(p, pairs):
    s=open(p,encoding='utf-8').read()
    for a,b in pairs:
        if a not in s: print("MISSING in",p,":",a[:60])
        s=s.replace(a,b)
    open(p,'w',encoding='utf-8',newline='\n').write(s)
sub('omml.py',[('''        elif t == "\\ ":
            self.add_text(nodes, " ", style if style != "math" else "roman")''','''        elif t == "\\ ":
            self.add_text(nodes, "\u00a0", style if style != "math" else "roman")''')])
sub('units.csv',[('S$M per t/h,M\ (t\ h^{-1})^{-1},money','S$M per t/h,M\ per\ t\ h^{-1},money'),
                 ('US$/MWth-h,/MW_{th}h,usmoney','US$/MWth-h,\ MW_{th}^{-1}\ h^{-1},usmoney')])
sub('build_report.py',[('''    def plain(chunk):
        if chunk:
            r = par.add_run(chunk.replace("~", "\u00a0"))
            r.bold = True if bold else None
''','''    def plain(chunk):
        for i, piece in enumerate(chunk.split("\n")):
            if i:
                par.add_run().add_break()
            if piece:
                r = par.add_run(piece.replace("~", "\u00a0"))
                r.bold = True if bold else None
''')])
sub('tables.py',[('head4 = ["BOAK, 25~[[y]]", "BOAK, 60~[[y]]", "FOAK, 25~[[y]]", "FOAK, 60~[[y]]"]','head4 = ["BOAK,\n25~[[y]]", "BOAK,\n60~[[y]]", "FOAK,\n25~[[y]]", "FOAK,\n60~[[y]]"]')])
sub('content/04_economics.md',[
 (r'\Delta C = \sum_{i} C_{i}, \quad', r'\Delta C = \sum_{i} {C_{i}}, \quad'),
 (r'\frac{\sum_{t} c_{x,t} \left(1 + r\right)^{-t}}{\sum_{t} m_{t} \left(1 + r\right)^{-t}}', r'\frac{\sum_{t} {c_{x,t} \left(1 + r\right)^{-t}}}{\sum_{t} {m_{t} \left(1 + r\right)^{-t}}}'),
 (r'\frac{\sum_{t} \Delta c_{t} \left(1 + r\right)^{-t}}{\sum_{t} A_{t} \left(1 + r\right)^{-t}}', r'\frac{\sum_{t} {\Delta c_{t} \left(1 + r\right)^{-t}}}{\sum_{t} {A_{t} \left(1 + r\right)^{-t}}}'),
 ('Gas is {{GAS_PRICE_S_GJ|f0|u}} [@fredgas].','Gas is {{GAS_PRICE_S_GJ|f0|u}} [@fredgas], and the plant saves {{NG_SAVED_B_MMSCFD|u}} against the unabated SMR.'),
 ('{fig:f_grid} and {tab:t_grid} hold the three size treatments at equal weight.','{tab:t_grid} and {fig:f_grid} hold the three size treatments at equal weight.'),
 ('!!figure f_grid\n\n!!table t_grid\n\n!!table t_unc\n','!!table t_grid\n\n!!table t_unc\n\n!!figure f_grid\n'),
 ('The row is indicative, since gas price, availability and scale differ','The row is indicative, since gas price and scale differ'),
])
sub('tables.py',[('caption="SMR plus CCS comparator, indicative only, since gas price, availability and scale differ."','caption="SMR plus CCS comparator, indicative only, since gas price and scale differ."')])
sub('content/01_problem.md',[
 ('A stripper regenerates the rich solvent, releasing concentrated CO₂ for compression, ship transport and offshore storage. The Idaho National Laboratory','A stripper regenerates the rich solvent, releasing concentrated CO₂ for compression and shipment to offshore storage. The Idaho National Laboratory'),
 ('and the remaining {{MARGIN_B_MWTH|u}} stays as spare with no revenue.\n\n!!figure f_pfd\n\nNatural gas','and the remaining {{MARGIN_B_MWTH|u}} stays as spare with no revenue.\n\nNatural gas'),
 ('and no measurement on syngas was found.\n\nThe PSA stays','and no measurement on syngas was found.\n\n!!figure f_pfd\n\nThe PSA stays'),
])
sub('content/02_feasibility.md',[
 ('contributes reactor modelling, safety analysis and public education [@r22,r23,r24]. National Environment Agency responsibilities, international regulatory cooperation and the Energy Market Authority\'s feasibility study add to the institutional basis [@r25,r26,r27]. A future project needs one coordinated licensing framework covering reactor operation, the nuclear-industrial heat interface and emergency preparedness.',
  'contributes reactor modelling and safety analysis alongside public education [@r22,r23,r24]. The National Environment Agency\'s responsibilities and the Energy Market Authority\'s feasibility study add to the institutional basis, supported by regulatory cooperation abroad [@r25,r26,r27]. A future project needs one licensing framework that spans the reactor and its industrial heat interface, with emergency preparedness built in.'),
 ('## 2.6 Risks, constraints and limitations','## 2.6 Risks and limitations'),
 ('The {{ETA_POWER_PCT|f1}}% power-conversion assumption, IHX performance and the transfer of INL\'s recycle to MEA all need integrated verification.','IHX performance and the transfer of INL\'s recycle to MEA need integrated verification, as does the {{ETA_POWER_PCT|f1}}% power-conversion assumption.'),
 ('Gas, hydrogen and CO₂ links to the island are unpriced.','Pipeline links to the island are unpriced.'),
])
sub('qa_report.py',[('''    bad_s = [str(t) for t in re.findall(r"(?:S\$|US\$)", "".join(f.read_text(encoding="utf-8") for f in (B / "content").glob("*.md")))]''','''    bad_s = [str(t) for t in re.findall(r"(?:S\$|US\$)", re.sub(r"\{\{[^}]+\}\}|\[\[[^\]]+\]\]", " ", "".join(f.read_text(encoding="utf-8") for f in (B / "content").glob("*.md"))))]''')])
