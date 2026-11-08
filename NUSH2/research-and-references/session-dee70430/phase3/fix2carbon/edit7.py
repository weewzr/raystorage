p='engine/plant/src/export.rs'
s=open(p,encoding='utf-8').read()
reps=[
('''fn ledger_line_text(m: &Model, id: &str, line: &str) -> String {
    if !m.carbon_params.independent() {
        return line.to_string();
    }
    match id {
        "GAS" => "Gas saved against the unabated reference (gas implied by the generic reference intensity)".to_string(),''',
'''fn ledger_line_text(m: &Model, src: CarbonSrc, id: &str, line: &str) -> String {
    if !m.carbon_params.independent() {
        return line.to_string();
    }
    match id {
        // A run on the anchored carbon (B_REC_FIX) keeps INL's gas.
        "GAS" if src == CarbonSrc::Plant => {
            "Gas saved against the unabated reference (gas implied by the generic reference intensity)"
                .to_string()
        }'''),
('''                                ledger_line_text(m, id, line),''','''                                ledger_line_text(m, b.c.src, id, line),'''),
]
for a,b in reps:
    assert s.count(a)==1, a
    s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
