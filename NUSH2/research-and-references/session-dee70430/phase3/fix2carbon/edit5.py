p='engine/wasm/src/registry.rs'
s=open(p,encoding='utf-8').read()
reps=[
('"Abatement cost at ladder step L3",',
 '"Abatement cost at ladder step L3, per tonne against the central reference (a generic reference intensity on the independent basis)",'),
('"Levelised abatement cost, discounted cash flow",',
 '"Levelised abatement cost, discounted cash flow, per tonne against the central reference (a generic reference intensity on the independent basis)",'),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
