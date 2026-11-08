p='engine/plant/src/export.rs'
s=open(p,encoding='utf-8').read()

a='''fn line_meta(id: &str) -> (&'static str, &'static str, &'static str, &'static str) {'''
b='''/// Line text of a ledger row. On the independent basis the three lines that
/// depend on the reference plant say which reference they price: the GAS line
/// the gas that the generic reference intensity implies, and the LOST_STEAM and
/// BASE_POWER lines the conventional plant of INL, a comparison input because
/// no source gives the duty of a plant-built reference. The legacy text is
/// unchanged.
fn ledger_line_text(m: &Model, id: &str, line: &str) -> String {
    if !m.carbon_params.independent() {
        return line.to_string();
    }
    match id {
        "GAS" => "Gas saved against the unabated reference (gas implied by the generic reference intensity)".to_string(),
        "LOST_STEAM" => "Lost steam export (INL conventional plant, comparison input)".to_string(),
        "BASE_POWER" => {
            "Baseline purchased power avoided (INL conventional plant, comparison input)".to_string()
        }
        _ => line.to_string(),
    }
}

fn line_meta(id: &str) -> (&'static str, &'static str, &'static str, &'static str) {'''
assert s.count(a)==1
s=s.replace(a,b,1)

a='''                                t(id),
                                t(line),
                                num(v),
                                num(lo_l[i].2),'''
b='''                                t(id),
                                ledger_line_text(m, id, line),
                                num(v),
                                num(lo_l[i].2),'''
assert s.count(a)==1
s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
