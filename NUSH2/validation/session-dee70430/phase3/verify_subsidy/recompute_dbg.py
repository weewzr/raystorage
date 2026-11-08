exec(open("recompute.py").read().split("rows = list(csv.DictReader")[0])
for st in ["FOAK","BOAK","10OAK_P46","10OAK_P47"]:
    for L in [25,60]:
        K, Y, L5sum, A = items(st, L)
        print(st, L, round(L5sum*1e6/A,6), round(lac_d(K,Y,L,A,"A",0.0,"PROGRESSIVE"),6), {k: round(v,3) for k,v in K.items()}, A)
