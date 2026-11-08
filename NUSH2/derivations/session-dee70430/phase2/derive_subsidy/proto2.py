import csv
import mpmath as mp
mp.mp.dps = 40
exec(open("C:/Users/Alf/AppData/Local/Temp/claude/f97511ab42c3e41a/dee70430-6901-4bdf-a7a6-7fbb5c9a2058/scratchpad/phase2/derive_subsidy/proto1.py").read().split("for st in")[0])
T=mp.mpf(100)
def buckets(C):
    out=[(-w, C*p/N)] if p>0 else []
    for k in range(w): out.append((k-w+1, C/N))
    return out
def priv_pv_dcf(C, G):
    # waterfall over reactor buckets, return PV at year 0 of private reactor flows and leftover grant
    S=mp.mpf(0); pv=mp.mpf(0)
    for t,b in buckets(C):
        pk=min(b,max(mp.mpf(0),S+b-G)); S+=b
        pv+=pk*(1+i)**(-t)
    left=max(mp.mpf(0),G-C)
    return pv,left
def h(u): return ((1+i)**(N*(1-u))-1)/(i*N)
res={}
for st in ['FOAK','BOAK','10OAK_P46','10OAK_P47']:
  for L in [25,60]:
    d=pt(st,L); a=d['a']
    C=d['REACTOR_CAP']/crf(L)
    Kihx=d['IHX']/crf(IL); Klic=d['LICENCE']/crf(L); Kpi=d['POWER_ISLAND']/crf(L)
    XB=Kihx+Klic+Kpi
    k=crf(L)*1e6/a
    L5_0=sum(d[x] for x in ['GAS','REACTOR_CAP','CAPTURE_CAP','CAPTURE_FOM','TS','REACTOR_OM','IHX','LICENCE','DECOM','IDC','POWER_ISLAND','RECYCLE_TRAIN','LOST_STEAM','BASE_POWER'])*1e6/a
    D_0=d['LAC_DCF']
    out=[]
    for name,X in [('A',0),('B',XB)]:
        E=C+X
        # progressive
        sL=(L5_0-T)/(k*(C*fL+X)); sD=(D_0-T)/(k*(C*fD+X))
        # close: ledger
        def lacL(s,X=X,E=E):
            G=s*E; u=G/C
            if u<=1: red=k*C*(fL-h(u))
            else: red=k*(C*fL+(G-C))
            return L5_0-red
        def lacD(s,X=X,E=E):
            G=s*E
            pv,left=priv_pv_dcf(C,G)
            return D_0-k*(C*fD-pv+left)
        def root(f):
            lo,hi=mp.mpf(0),mp.mpf(1)
            if f(hi)>=T: return None
            for _ in range(200):
                mid=(lo+hi)/2
                if f(mid)<T: hi=mid
                else: lo=mid
            return hi
        out.append((name,sL,sD,root(lacL),root(lacD)))
    # dated-flow bound A
    sd=(L5_0-T)/(k*C*(1+i)**N)
    print(st,L,'L5_0',mp.nstr(L5_0,7),'dcf0',mp.nstr(D_0,7),' '.join(f"{n}: prog L5 {mp.nstr(a_,4)} DCF {mp.nstr(b_,4)} | close L5 {mp.nstr(c_,4) if c_ is not None else None} DCF {mp.nstr(e_,4) if e_ is not None else None}" for n,a_,b_,c_,e_ in out),'dated',mp.nstr(sd,4))
