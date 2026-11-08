import math
i=0.08
# IEAGHG convention: spend at year end, interest on prior year-end balance (compounded balance), 20/45/35
sp=[0.20,0.45,0.35]; bal=0; idc=0
for s in sp:
    ii=bal*i; idc+=ii; bal=bal+ii+s
print("IEAGHG-type IDC fraction of TPC (3y 20/45/35, 8%):",round(idc,5),"balance",round(bal,5))
# same convention, 5.5y repo: ledger closed form and DCF buckets
N=5.5
f_ledger=((1+i)**N-1)/(i*N)
b=[(-5,0.5/5.5)]+[(k,1/5.5) for k in range(-4,1)]
f_dcf=sum(v*(1+i)**(-y) for y,v in b)
f_cont=((1+i)**N-1)/(math.log(1+i)*N)
print("ledger",round(f_ledger,5),"dcf",round(f_dcf,5),"cont",round(f_cont,5))
print("IDC increment ratio ledger/cont",round((f_ledger-1)/(f_cont-1),4),"dcf/ledger increment",round((f_dcf-1)/(f_ledger-1),4))
print("grant at close compounding to COD: t=-5:",round((1+i)**5,5)," t=-5.5:",round((1+i)**5.5,5), "ratio",round((1+i)**0.5,5))
# NETL-type simple interest, full-year (funds Jan 1) for 5 equal years vs ledger convention
# 5-year equal spend: cost of funding = sum_{n=1..5} (y-n+1)*r*(1/5)
y=5; fn=1+sum((y-n+1)*i/5 for n in range(1,y+1)); print("NETL-type simple, 5y equal, 8%:",round(fn,4))
# year-end compounded, 5 equal years (ledger style integer N=5)
f5=((1+i)**5-1)/(i*5); print("year-end compounded 5y equal:",round(f5,4))
# affine check: LAC(s)=LAC0 - s*G ; s* = (LAC0-100)/(LAC0-LAC1)
LAC0,LAC1=150.0,60.0  # placeholders purely to show algebra, not repo numbers
print("illustrative s*",(LAC0-100)/(LAC0-LAC1))
