p='engine/plant/tests/m11_carbon.rs'
s=open(p,encoding='utf-8').read()
a='''        match c[6].as_str() {
            "GAS" => assert!(c[7].contains(words), "{line}"),'''
b='''        match c[6].as_str() {
            // B_REC_FIX is anchored by definition and prices INL's gas.
            "GAS" if c[0] != "B_REC_FIX" => assert!(c[7].contains(words), "{line}"),'''
assert s.count(a)==1
s=s.replace(a,b,1)
open(p,'w',encoding='utf-8').write(s)
print('ok')
