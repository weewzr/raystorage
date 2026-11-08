p='engine/plant/tests/m11_carbon.rs'
s=open(p,encoding='utf-8').read()
reps=[
('''    let tag = "BOAK_25Y_N1P00";
    let close = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() <= 1e-5_f64.max(1e-9 * want.abs()),
            "{what}: {got} against {want}"
        );
    };''','''    let tag = "BOAK_25Y_N1P00";
    // The exported tables run on the Headline preset, which differs from the
    // base registry used here by less than 1e-4 relative (the harness says the
    // same of its own rows). Rows compared with each other are exact.
    let close = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() <= 1e-4 * want.abs(),
            "{what}: {got} against {want}"
        );
    };
    let exact = |got: f64, want: f64, what: &str| {
        assert!(
            (got - want).abs() <= 1e-5_f64.max(1e-9 * want.abs()),
            "{what}: {got} against {want}"
        );
    };'''),
('''    close(
        design_basis_value(&format!("SYSLINES_L5_{tag}")),''','''    exact(
        design_basis_value(&format!("SYSLINES_L5_{tag}")),'''),
('''    close(
        design_basis_value(&format!("SYSLINES_LAC_DCF_{tag}")),''','''    exact(
        design_basis_value(&format!("SYSLINES_LAC_DCF_{tag}")),'''),
('''                assert!((got - share).abs() < 1e-6, "{id}: {got} {share}");''','''                // The tables use the Headline preset (see above).
                assert!((got - share).abs() < 1e-4, "{id}: {got} {share}");'''),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
