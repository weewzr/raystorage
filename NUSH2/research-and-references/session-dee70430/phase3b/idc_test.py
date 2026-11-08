tpc=170.952e6; sp=854760.0; ow=11966640.0
su=3419040+1478446+1074356+163214
sh=(0.20,0.45,0.35)
def run(n_int_years, owner_in_y1=True, include_su=False, r=0.08, spend_start=False):
    bal=0.0; tot=0.0
    for y in range(n_int_years):
        if y<3:
            s=sh[y]*(tpc+sp)+(ow if (y==0 and owner_in_y1) else 0)
        else:
            s=su if include_su else 0.0
        interest=bal*r
        bal+=interest+s
        tot+=interest
    return tot
for n in (3,4):
    for su_ in (False,True):
        print(n, su_, round(run(n,include_su=su_)))
# spending at start of year (interest on opening balance incl. same-year spend)
def run2(n, r=0.08):
    bal=0.0; tot=0.0
    for y in range(n):
        s=sh[y]*(tpc+sp)+(ow if y==0 else 0) if y<3 else 0
        bal+=s
        i=bal*r
        bal+=i; tot+=i
    return tot
print("start-of-year spend, 3 y:", round(run2(3)), " 4 y:", round(run2(4)))
print("target 32941928")
