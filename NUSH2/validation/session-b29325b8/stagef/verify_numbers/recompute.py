"""Recompute MATURE_60Y, STEPDOWN and COFUNDING from Economic_Ledger.csv lines."""
import csv, math, sys
R = sys.argv[1]
led = list(csv.DictReader(open(R + '/results/Economic_Ledger.csv', encoding='utf-8')))
sens = list(csv.DictReader(open(R + '/results/Sensitivity_Results.csv', encoding='utf-8')))

def rows(stage, life):
    return [r for r in led if r['run_id'] == 'B_REC_F' and r['stage'] == stage and r['life_y'] == str(life)
            and r['n'] == '1.0000' and r['reactor_charged_mwth'] == '210.00']

def line(rs, lid):
    return [r for r in rs if r['line_id'] == lid][0]

out = {}
for stage, life in [('BOAK', 25), ('BOAK', 60), ('10OAK_P46', 25), ('10OAK_P46', 60), ('FOAK', 25), ('FOAK', 60)]:
    rs = rows(stage, life)
    lv = [r for r in rs if r['level'].startswith('L')]
    total = sum(float(r['value_sgdm_y']) for r in lv)
    a = float(lv[0]['abatement_t_y'])
    l5 = total * 1e6 / a
    dcf = float(line(rs, 'LAC_DCF')['dcf_value'])
    print(f'{stage:10s} {life}y  sum lines {total:.6f} S$M/y  /A {a:.4f} -> L5 {l5:.6f} (cum col {lv[-1]["cum_s_per_t"]})  DCF {dcf:.6f}  nlines {len(lv)}')
    out[(stage, life)] = dict(rs=rs, l5=l5, dcf=dcf, a=a)

# gas sensitivity: GAS and LOST_STEAM scale linearly with gas price (economics.rs:287-348)
m = out[('10OAK_P46', 60)]
gas = float(line(m['rs'], 'GAS')['value_sgdm_y'])
steam = float(line(m['rs'], 'LOST_STEAM')['value_sgdm_y'])
slope = (gas + steam) / 15.0 * 1e6 / m['a']  # S$/t per S$/GJ
print('slope S$/t per S$/GJ', slope)
for meth, base in [('L5', m['l5']), ('DCF', m['dcf'])]:
    need = 15 + (base - 96) / (-slope)
    print(f'MATURE {meth}: central {base:.6f} at20 {base + slope*5:.6f} at20.7 {base + slope*5.7:.6f} gas_needed {need:.6f}')
print('printed', math.ceil(max(15 + (out[("10OAK_P46",60)]["l5"]-96)/(-slope), 15 + (out[("10OAK_P46",60)]["dcf"]-96)/(-slope))*10)/10)

# cofunding: share s of reactor capital (and its IDC) such that cost = 96
for life in (25, 60):
    b = out[('BOAK', life)]
    rc = float(line(b['rs'], 'REACTOR_CAP')['value_sgdm_y'])
    idc = float(line(b['rs'], 'IDC')['value_sgdm_y'])
    per_t = (rc + idc) * 1e6 / b['a']
    s_l5 = (b['l5'] - 96) / per_t
    gap = float(line(b['rs'], 'GAP_REACTOR_CAP_WITH_IDC')['dcf_value'])
    s_dcf = (b['dcf'] - 96) / (per_t + gap)
    # overnight capital: rc / crf
    crf = 0.08 * 1.08**life / (1.08**life - 1)
    cap = rc / crf
    print(f'COFUND BOAK {life}y: L5 share {s_l5:.6f} S$M {s_l5*cap:.3f} | DCF share {s_dcf:.6f} S$M {s_dcf*cap:.3f} | capital {cap:.4f}')

print('--- Sensitivity rows')
for r in sens:
    if r['sweep_id'] in ('MATURE_60Y', 'STEPDOWN', 'COFUNDING'):
        print(r['sweep_id'], r['run_id'], r['metric'], r['test_value'], r['result'])
