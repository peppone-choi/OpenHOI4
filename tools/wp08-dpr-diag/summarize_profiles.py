from pathlib import Path
import json,collections,sys
folder=Path(sys.argv[1]);out=[]
for p in sorted(folder.glob('*profile.json')):
    j=json.loads(p.read_text());rpc=j['rpc'];phases=[]
    for a,b in zip(j['phases'],j['phases'][1:]):phases.append({'stage':a['stage'],'ms':b['ms']-a['ms'],'rpc':b['rpcCount']-a['rpcCount']})
    methods=collections.Counter(r['type']+'.'+r['method'] for r in rpc)
    reads=collections.Counter(r.get('expressionKind') or r['method'] for r in rpc if r['type'] in ['Frame','ElementHandle'] and r['method'] in ['evaluateExpression','evaluateExpressionHandle','getAttribute','textContent','boundingBox','expect','waitForSelector'])
    row={'name':p.name,'totalMs':j['phases'][-1]['ms'],'rpcCount':len(rpc),'rpcSumMs':sum(r['ms'] for r in rpc),'pngDecodeSumMs':sum(r['ms'] for r in j['cpu']),'phases':phases,'methods':dict(methods),'reads':dict(reads),'largest':sorted(rpc,key=lambda r:r['ms'],reverse=True)[:8]};out.append(row)
    print(row['name'],'total',round(row['totalMs']),'RPC',row['rpcCount'],'PNGms',round(row['pngDecodeSumMs']),'methods',methods.most_common(7));print('phases',[(x['stage'],round(x['ms']),x['rpc']) for x in phases])
(folder/'profile-summary.json').write_bytes((json.dumps(out,indent=2)+'\n').encode())
