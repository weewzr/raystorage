import json
P={p['id']:p['value'] for p in json.load(open("C:/Users/Alf/AppData/Local/Temp/claude/f97511ab42c3e41a/dee70430-6901-4bdf-a7a6-7fbb5c9a2058/scratchpad/audit/provenance-critic/params.json"))}
LB=0.45359237;ST=0.90718474;SCF=379.49;MWCO2=44.0095
def st_d_to_kmol_h(x): return x*ST*1000/24/MWCO2
def mm(x): return x*1e6/SCF/24*LB
def kmol_to_t_h(k): return k*MWCO2/1000
def run(over={}):
    p=dict(P);p.update(over)
    avail=p['AVAILABILITY'];hours=avail*8760;days=avail*365
    h2a=p['H2_A']*LB
    s_b=p['H2_B']*p['RHO_H2_NORMAL']/h2a
    cpm=sum(p['C_PER_MOL_NG_'+k]*n for k,n in [('CH4',1),('C2',2),('C3',3),('C4',4),('C5',5),('C6',6),('CO2',1)])
    f_c6=mm(p['NG_CASE6'])*cpm
    p_c=mm(p['H2_A_MMSCFD'])*p['CH4_IN_H2']
    cap6=st_d_to_kmol_h(p['CO2_CAPTURED_C6'])
    phi=p['PHI_RECYCLE'];eta=p['ETA_MEA']
    t6=(cap6/p['ETA_SELEXOL'])/phi
    g6=t6-cap6-p_c
    f6=(t6-f_c6)/g6
    t=(f_c6-f6*p_c)/(1-f6*(1-eta*phi))
    cap=eta*phi*t
    g=t-cap-p_c
    burn=(1-f6)*g
    s=s_b
    tpy=lambda k: kmol_to_t_h(k)*hours*s
    base=p['CO2_CONV_NOCC']*ST*days*s
    atm=tpy(burn);captured=tpy(cap)
    h2_t_y=h2a*s*hours/1000
    return dict(s=s,f6=f6,t=t,cap=cap,burn=burn,base=base,atm=atm,captured=captured,A=base-atm,h2=h2_t_y,ci=atm/h2_t_y,bi=base/h2_t_y)
if __name__=='__main__':
    r=run();print(r)
