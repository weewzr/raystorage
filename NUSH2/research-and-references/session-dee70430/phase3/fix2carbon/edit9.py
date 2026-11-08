p='engine/plant/src/sens.rs'
s=open(p,encoding='utf-8').read()
a0=s.index('        let by_group: [(&str, Point<\'_>, BaselineKind); 6] = [')
a1=s.index('        let mut gas = Vec::new();')
new='''        let by_group: [(&str, Point<'_>, BaselineKind); 6] = [
            (
                "BASELINE IEAGHG",
                Point {
                    baseline: Baseline::Ieaghg,
                    ..CENTRAL
                },
                BaselineKind::Ieaghg,
            ),
            (
                "BASELINE INL",
                Point {
                    baseline: Baseline::Inl,
                    ..CENTRAL
                },
                BaselineKind::Inl,
            ),
            (
                "BASELINE Cho",
                Point {
                    baseline: Baseline::Cho,
                    ..CENTRAL
                },
                BaselineKind::Cho,
            ),
            (
                "CARBON_BASELINE 0",
                Point {
                    params: &[("CARBON_BASELINE", 0.0)],
                    ..CENTRAL
                },
                BaselineKind::Ieaghg,
            ),
            (
                "CARBON_BASELINE 1",
                Point {
                    params: &[("CARBON_BASELINE", 1.0)],
                    ..CENTRAL
                },
                BaselineKind::Inl,
            ),
            (
                "CARBON_BASELINE 2",
                Point {
                    params: &[("CARBON_BASELINE", 2.0)],
                    ..CENTRAL
                },
                BaselineKind::Cho,
            ),
        ];
'''
s=s[:a0]+new+s[a1:]
open(p,'w',encoding='utf-8').write(s)
print('ok')
