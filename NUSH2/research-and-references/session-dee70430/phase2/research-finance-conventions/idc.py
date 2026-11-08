import math
i=0.08; N=5.5
f_ledger=((1+i)**N-1)/(i*N)
# DCF buckets: part year 0.5/5.5 at -5, then 1/5.5 at -4..0 (uniform_spend), compounded to year 0
flows=[(-5,0.5/5.5)]+[(k,1/5.5) for k in range(-4,1)]
f_dcf=sum(v*(1+i)**(-y) for y,v in flows)
# continuous-spend, annual-compounding: integral over [0,N] of (1+i)^(N-t) dt / N
f_cont=((1+i)**N-1)/(math.log(1+i)*N)
# all at close: (1+i)^N
f_close=(1+i)**N
# DCF dated close at -5.5 treated as lump
print("ledger",f_ledger,"dcf buckets",f_dcf,"continuous",f_cont,"lump at close",f_close)
# mid-year / start-of-year bucket variants
f_start=sum(((1+i)**(N-t)) for t in [0.0+k for k in range(5)])/N  # not exact
crf=lambda n: i/(1-(1+i)**-n)
print("crf25",crf(25),"crf60",crf(60))
# ledger vs DCF reactor-only annualised ratio on a 25y life
for L in (25,60):
    ann=sum((1+i)**-y for y in range(1,L+1))
    print(L,"ledger PV/yr basis: C*f*CRF -> PV C*f ; DCF PV factor",f_dcf,"ratio dcf/ledger",f_dcf/f_ledger)
