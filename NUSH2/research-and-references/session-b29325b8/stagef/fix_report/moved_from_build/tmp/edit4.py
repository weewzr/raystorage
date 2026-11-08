s=open('build_report.py',encoding='utf-8').read()
a='''    pf.line_spacing_rule = WD_LINE_SPACING.ONE_POINT_FIVE
    pf.space_after = Pt(6)'''
b='''    # Word's 1.5 lines for Times New Roman 12 pt is 20.7 pt. Exact spacing stops inline math
    # from opening individual lines, per the plan's line-height fallback.
    pf.line_spacing = Pt(20.7)
    pf.line_spacing_rule = WD_LINE_SPACING.EXACTLY
    pf.space_after = Pt(6)'''
assert a in s
s=s.replace(a,b)
open('build_report.py','w',encoding='utf-8',newline='\n').write(s)
s=open('qa_report.py',encoding='utf-8').read()
a='record("3b", "Line height with inline math", "PASS" if modal and abs(modal - 20.7) <= 0.5 else "FAIL",'
b='record("3b", "Line height with inline math", "PASS" if modal and abs(modal - 20.7) <= 0.5 and off == 0 else "FAIL",'
assert a in s
s=s.replace(a,b)
open('qa_report.py','w',encoding='utf-8',newline='\n').write(s)
