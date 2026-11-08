s=open('qa_report.py',encoding='utf-8').read()
a='''    off = sum(c for d, c in pitch.items() if modal and abs(d - modal) > 0.5)'''
b='''    # Under exact spacing a line cannot grow, so a gap above 21.5 pt is a paragraph or table boundary.
    boundary = sum(c for d, c in pitch.items() if d >= 21.5) if rule == "exact" else 0
    off = sum(c for d, c in pitch.items() if modal and abs(d - modal) > 0.5 and not (rule == "exact" and d >= 21.5))'''
assert a in s
s=s.replace(a,b)
a='''f"modal pitch {modal} pt, {off} of {total_pitch} pitches more than 0.5 pt from it, largest {max(pitch) if pitch else None}: {sorted(pitch.items())}")'''
b='''f"modal pitch {modal} pt, {off} of {total_pitch} pitches more than 0.5 pt from it inside paragraphs, {boundary} at paragraph or table boundaries")'''
assert a in s
s=s.replace(a,b)
open('qa_report.py','w',encoding='utf-8',newline='\n').write(s)
