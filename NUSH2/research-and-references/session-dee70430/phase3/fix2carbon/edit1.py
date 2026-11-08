p='engine/plant/src/export.rs'
s=open(p,encoding='utf-8').read()
reps=[
('''                "Net abatement after transport and storage against the central reference, {}",
                m.carbon_params.baseline.id()''',
'''                "Net abatement after transport and storage against the generic reference intensity (central), {}",
                m.carbon_params.baseline.id()'''),
('''                "Unabated reference, central: {} times the annual hydrogen",''',
'''                "Unabated reference, central generic reference intensity: {} times the annual hydrogen",'''),
('''            "Direct abatement against the central reference, before transport and storage",''',
'''            "Direct abatement against the generic reference intensity (central), before transport and storage",'''),
('''        let tag = if row.id == ab.central {
            " (central)"
        } else {
            ""
        };''',
'''        let tag = if row.id == ab.central {
            " (central, generic reference intensity)"
        } else {
            ""
        };'''),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print("ok")
