import csv, json, sys
f = open('Economic_Ledger.csv', encoding='utf8', newline='')
rd = csv.reader(f)
hdr = next(rd)
ix = {h: i for i, h in enumerate(hdr)}
groups = {}
for no, r in enumerate(rd, 2):
    key = (r[ix['run_id']], r[ix['stage']], int(r[ix['life_y']]), r[ix['n']], r[ix['reactor_charged_mwth']])
    g = groups.setdefault(key, {'sum': 0.0, 'l5_line': None, 'dcf_line': None})
    if r[ix['level']] in ('L0', 'L1', 'L2', 'L3', 'L4', 'L5'):
        g['sum'] += float(r[ix['value_sgdm_y']])
        if r[ix['level']] == 'L5':
            g['l5_line'] = no
            g['l5_cum'] = float(r[ix['cum_s_per_t']])
            g['ab'] = float(r[ix['abatement_t_y']])
    if r[ix['line_id']] == 'LAC_DCF':
        g['dcf_line'] = no
        g['dcf'] = float(r[ix['dcf_value']])
maxdiff = 0
out = []
for k, g in groups.items():
    d = g['sum'] * 1e6 / g['ab'] - g['l5_cum']
    maxdiff = max(maxdiff, abs(d))
    out.append([*k, g['l5_line'], g['l5_cum'], g['dcf_line'], g.get('dcf'), g['ab']])
print('groups', len(groups), 'max |sum/ab - cum| =', maxdiff)
with open(sys.argv[1], 'w', newline='') as o:
    w = csv.writer(o)
    w.writerow(['run_id', 'stage', 'life_y', 'n', 'charged_mwth', 'l5_csv_line', 'L5_s_per_t', 'dcf_csv_line', 'LAC_DCF_s_per_t', 'abatement_t_y'])
    w.writerows(out)
for o in out:
    if o[0] != 'B_REC_F' and o[2] in (25, 60) and o[1] in ('FOAK', 'BOAK'):
        print(o[0], o[1], o[2], o[4], round(o[6], 3), round(o[8], 3) if o[8] else None, round(o[9], 1))
