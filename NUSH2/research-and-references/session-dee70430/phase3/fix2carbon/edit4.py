p='engine/wasm/src/registry.rs'
s=open(p,encoding='utf-8').read()
reps=[
('"Annual abatement, net of transport and storage, central reference",',
 '"Annual abatement, net of transport and storage, against the central reference (a generic reference intensity on the independent basis)",'),
('"Annual direct abatement, before transport and storage",',
 '"Annual direct abatement, before transport and storage, against the central reference (a generic reference intensity on the independent basis)",'),
('"Abatement cost, L5 ledger, selected stage, life and exponent",',
 '"Abatement cost, L5 ledger, selected stage, life and exponent, per tonne against the central reference (a generic reference intensity on the independent basis)",'),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
