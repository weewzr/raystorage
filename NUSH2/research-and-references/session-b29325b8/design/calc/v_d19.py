import re,sys
t=open('findings/mea_txt/vinj_param_b.txt',encoding='utf8').read()
i=t.index('Table D.19'); seg=t[i:]
j=seg.index('wtMEA,rich'); seg=seg[seg.index('[mol CO2/mol MEA]',j):]
seg=seg[:seg.index('PDF PAGE 39')] if 'PDF PAGE 39' in seg else seg
nums=re.findall(r'(?<![\d.])(\d{1,3})\s+(\d\.\d+)\s+(\d{2,3}(?:\.\d+)?)\s+(\d{2}(?:\.\d+)?)\s+(\d{2}(?:\.\d+)?)\s+(0\.\d+)\s+(0\.\d+)',seg)
rows=[(int(a),float(b),float(c),float(d),float(e),float(f),float(g)) for a,b,c,d,e,f,g in nums]
print(len(rows), rows[-1])
w=[r for r in rows if 88<=r[2]<=92]
print('window',len(w),min(r[1] for r in w),max(r[1] for r in w),sorted(r[1] for r in w)[len(w)//2])
print('lowest',sorted(rows,key=lambda r:r[1])[:5])
print('all n',len(rows),'median',sorted(r[1] for r in rows)[len(rows)//2],'range',min(r[1] for r in rows),max(r[1] for r in rows))
