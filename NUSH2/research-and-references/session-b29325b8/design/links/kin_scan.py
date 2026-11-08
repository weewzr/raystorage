import re,glob,os,sys,io
sys.stdout=io.TextIOWrapper(sys.stdout.buffer,encoding='utf-8',errors='replace')
sp=r"C:/Users/Alf/AppData/Local/Temp/claude/C--Ray-Claude-Ray-Debug-VS-Code/b29325b8-d1b9-49cd-bfe6-262261d7ffef/scratchpad"
dirs=['findings/inv_txt','findings/mea_thermo_txt','findings/mea_txt','findings/smr_wgs_txt','findings/htgr_txt','findings/econ_txt','findings/inl_txt']
files=[]
for d in dirs:
    files+=glob.glob(os.path.join(sp,d,'*.txt'))
print(len(files))
pats={
 'LTS kin':r'Cu/ZnO|Cu-ZnO|CuO/ZnO|Cu/Zn|copper.zinc|low.temperature shift|Moe |Choi and Stenger|Rase\b|Ayastuy|Ovesen',
 'gas sol':r'solubility of (hydrogen|H2|methane|CH4|carbon monoxide|CO\b)|hydrogen solubility|H2 solubility|methane solubility|CH4 solubility',
 'heatcap':r'heat capacit(y|ies) of .{0,40}(MEA|monoethanolamine|loaded|amine)|Weiland.{0,80}heat|Amundsen|Hilliard.{0,60}(capacity|Cp)',
 'Hou':r'Hou and Hughes|Hou, K|Hou K',
 'Chinen':r'Chinen',
 'VersteegJCED':r'J\.? ?Chem\.? ?Eng\.? ?Data.{0,30}33|33.{0,8}29.{0,4}34',
 'SongChen':r'Song.{0,12}Chen',
 'YanChen':r'Yan.{0,12}Chen',
 'ZhangChen':r'Zhang.{0,12}Chen',
 'Chinchen':r'Chinchen',
 'JANAF':r'JANAF',
 'Antoine':r'Antoine',
 'Twigg':r'Twigg',
}
seen={}
for f in files:
    if not f.endswith('_full.txt') and 'inv_txt' in f: continue
    t=open(f,encoding='utf-8',errors='replace').read()
    t=re.sub(r'\s+',' ',t)
    for k,p in pats.items():
        for m in list(re.finditer(p,t,re.I))[:3]:
            s=t[max(0,m.start()-110):m.end()+170]
            seen.setdefault(k,[]).append((os.path.basename(os.path.dirname(f))+'/'+os.path.basename(f),s))
for k,v in seen.items():
    print('=====',k,len(v))
    for a,b in v[:9]:
        print(' -',a,'::',b)
