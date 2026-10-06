import runpy,json,copy,pathlib
v=runpy.run_path('target/wp11-verify2/reference.py');globals().update({k:v[k] for k in ['R','O','P','PACK','C','trial','recompute','verify','base']})
for bits in [-2**63,2**63-1,-1,0,1]:
 d=copy.deepcopy(base);t=d['world']['inputs']['states'][1];t['base']=0;t['modifiers']=[{'source':'edge.bits','target_stat':'infrastructure','op':'Add','value':bits,'expires':None}];recompute(d);assert t['infrastructure']==bits;trial('independent-Fx-'+str(bits),d,True)
print('five signed Fx raw-bit extremes accepted; full DTO, ledger and canonical verified')
