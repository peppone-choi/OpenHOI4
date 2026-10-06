import run_checks as m,subprocess,json,re,urllib.request,hashlib,time
port=int(__import__('sys').argv[1]);label=__import__('sys').argv[2]
tcp=subprocess.check_output(['netstat','-ano','-p','tcp']).decode();lines=[l for l in tcp.splitlines() if re.search(r'127\.0\.0\.1:'+str(port)+r'\s',l) and 'LISTENING' in l];assert len(lines)==1,lines
pid=int(lines[0].split()[-1])
script=f"Get-CimInstance Win32_Process -Filter 'ProcessId={pid}' | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress"
proc=json.loads(subprocess.check_output(['powershell','-NoProfile','-Command',script]).decode('utf-8-sig'))
assert str(m.ROOT).lower() in proc['ExecutablePath'].lower(),proc
base=f'http://127.0.0.1:{port}'
def get(path):
 with urllib.request.urlopen(base+path) as r:return {'status':r.status,'headers':dict(r.headers),'bytes':r.read()}
html=get('/');assets=re.findall(r'(?:src|href)="(/assets/[^"]+)"',html['bytes'].decode());evidence=[]
for path in assets:
 r=get(path);local=m.ROOT/'client/dist'/path.lstrip('/');assert r['bytes']==local.read_bytes()
 evidence.append({'path':path,'status':r['status'],'length':len(r['bytes']),'sha256':hashlib.sha256(r['bytes']).hexdigest(),'local':str(local)})
meta=get('/maps/testland/metadata') if label!='m0-runtime' else {'bytes':b'{}'};meta=json.loads(meta['bytes']);index=get('/maps/testland/index.bin?pack='+meta['pack_hash']) if meta else {'bytes':b'','headers':{}}
data={'time':m.now(),'head':m.git('rev-parse','HEAD'),'port':port,'netstat':lines,'process':proc,'exeSHA':hashlib.sha256(__import__('pathlib').Path(proc['ExecutablePath']).read_bytes()).hexdigest(),'cwdEvidence':'Playwright webServer command spawned in '+str(m.ROOT/'client')+'; resolved ../target/debug executable and ../data/packs','html200':html['status'],'assets':evidence,'metadata':meta,'indexLength':len(index['bytes']),'indexSHA256':hashlib.sha256(index['bytes']).hexdigest(),'indexHeaders':index['headers']}
(m.OUT/(label+'.json')).write_text(json.dumps(data,indent=2));print(json.dumps(data,indent=2))
