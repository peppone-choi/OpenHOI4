import pathlib,struct,zlib,math,json,subprocess,random,hashlib
BASE=pathlib.Path.cwd()/'target/wp07-independent'; F=BASE/'fixtures'; F.mkdir(exist_ok=True)
IDS=[0,7,257,65535]; COLORS=[[3,13,23],[5,15,25],[7,17,27],[9,19,29]]
GRID=[0,0,7,7,257,257,257, 0,7,7,257,257,65535,65535, 0,0,7,257,257,65535,65535, 0,0,7,7,257,257,65535, 0,7,7,7,257,257,65535]
ROWS=['0,3,13,23,land,plain,true,false','7,5,15,25,land,plain,false,false','257,7,17,27,sea,water,false,false','65535,9,19,29,lake,water,false,false']
CSV='id,r,g,b,kind,terrain,coastal,island\n'+'\n'.join(ROWS)+'\n'
STATES='''[[state]]
id=9
name_key="south"
provinces=[7]
population=456
resources={ore=8}
buildings={mill=2}
infrastructure=3
[[state]]
id=0
name_key="north"
provinces=[0]
population=123
resources={ore=0}
buildings={mill=0}
infrastructure=0
[[victory_point]]
province=65535
points=2
name_key="lake_vp"
[[victory_point]]
province=0
points=1
name_key="land_vp"
[[victory_point]]
province=257
points=4
name_key="sea_vp"
'''
def chunk(t,b):return struct.pack('>I',len(b))+t+b+struct.pack('>I',zlib.crc32(t+b)&0xffffffff)
def png(w,h,grid,colors=COLORS,ids=IDS,depth=8,kind=2,apng=False):
    ch={0:1,2:3,3:1,4:2,6:4}[kind]; data=b''
    for y in range(h):
        row=b''
        for i in grid[y*w:(y+1)*w]:
            rgb=colors[ids.index(i)]
            row+=bytes((rgb+[255])[:ch]) if depth==8 else b''.join(struct.pack('>H',x*257) for x in rgb[:ch])
        data+=b'\x00'+row
    out=b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',w,h,depth,kind,0,0,0))
    if kind==3:out+=chunk(b'PLTE',bytes(sum(colors,[])))
    if apng:out+=chunk(b'acTL',struct.pack('>II',1,0))+chunk(b'fcTL',struct.pack('>IIIIIHHBB',0,w,h,0,0,1,1,0,0))
    return out+chunk(b'IDAT',zlib.compress(data))+chunk(b'IEND',b'')
CASES=[]
def make(name,grid=GRID,w=7,h=5,csv=CSV,states=STATES,scale='1.25',over='a,b,kind\n',colors=COLORS,ids=IDS,err=None,warnings=None):
    p=F/name; (p/'maps/probe').mkdir(parents=True,exist_ok=True);(p/'common').mkdir(exist_ok=True)
    for file,text in [('terrain','[[terrain]]\nid="plain"\n[[terrain]]\nid="water"\n'),('resources','[[resource]]\nid="ore"\n'),('buildings','[[building]]\nid="mill"\n')]: (p/f'common/{file}.toml').write_text(text)
    r=p/'maps/probe'
    for file,text in [('provinces.csv',csv),('states.toml',states),('regions.toml',f'km_per_pixel={scale}\n'),('adjacency_overrides.csv',over)]: (r/file).write_bytes(text.encode())
    (r/'provinces.png').write_bytes(png(w,h,grid,colors,ids))
    CASES.append(dict(name=name,path=str(p),grid=grid,w=w,h=h,ids=ids,colors=colors,scale=scale,over=over,err=err,warnings=warnings));return p
make('irregular')
make('reverse_crlf',csv='id,r,g,b,kind,terrain,coastal,island\r\n'+'\r\n'.join(reversed(ROWS))+'\r\n')
make('overrides',over='a,b,kind\n7,0,river_small\n7,257,river_large\n257,65535,impassable\n0,65535,strait\n')
make('unknown_rgb',err=('provinces.png','unknown RGB',(1,1)))
p=F/'unknown_rgb/maps/probe/provinces.png'; cc=COLORS.copy();cc[0]=[2,12,22];p.write_bytes(png(7,5,GRID,cc))
# Independent negative mutations each starts from fresh fixture.
for name,old,new,msg,loc in [
 ('duplicate_id',ROWS[1],ROWS[1].replace('7,','0,',1),'duplicate province id',(3,1)),
 ('duplicate_rgb','7,5,15,25','7,3,13,23','duplicate province RGB',(3,3)),
 ('terrain_ref','land,plain,true','land,void,true','unknown terrain',(2,16)),
 ('id_overflow','0,3,13,23','65536,3,13,23','province id',(2,1)),
 ('rgb_overflow','0,3,13,23','0,256,13,23','red',(2,3)),
 ('kind','land,plain,true','bogus,plain,true','province kind',(2,11)),
 ('boolean','plain,true','plain,1','coastal',(2,22)),
 ('csv_quote','plain,true','"plain",true','unquoted',(2,1)),
 ('csv_column','plain,true,false','plain,true','unquoted',(2,1)),
 ('island_bool','true,false','true,1','island',(2,27))]:
  make(name,csv=CSV.replace(old,new),err=('provinces.csv',msg,loc))
make('crlf_error',csv=CSV.replace('land,plain,true','land,void,true').replace('\n','\r\n'),err=('provinces.csv','unknown terrain',(2,16)))
make('missing_png_color',csv=CSV+'123,22,33,44,sea,water,false,false\n',err=('provinces.csv','no bitmap pixels',None))
make('single_pixel',grid=[65535]+[257 if x==65535 else x for x in GRID[1:]],err=('provinces.png','1-pixel province',None))
dis=GRID.copy();dis[33]=0
make('disconnected',grid=dis,warnings=[0])
make('island_exception',grid=dis,csv=CSV.replace('land,plain,true,false','land,plain,true,true'),warnings=[])
make('island_still_single',grid=[65535]+[257 if x==65535 else x for x in GRID[1:]],csv=CSV.replace('lake,water,false,false','lake,water,false,true'),err=('provinces.png','1-pixel province',None))
for name,old,new,msg in [
 ('missing_land','provinces=[7]','provinces=[0]','multiple states'),
 ('unowned_land','provinces=[0]','provinces=[7]','multiple states'),
 ('repeated_land','provinces=[7]','provinces=[7,7]','repeatedly'),
 ('sea_state','provinces=[7]','provinces=[7,257]','only land'),
 ('lake_state','provinces=[7]','provinces=[7,65535]','only land'),
 ('state_unknown','provinces=[7]','provinces=[7,999]','unknown province'),
 ('state_duplicate','id=9','id=0','duplicate state'),
 ('population_negative','population=456','population=-1','population'),
 ('population_type','population=456','population=1.5','expected i64'),
 ('resource_ref','ore=8','wrong=8','unknown resource'),
 ('resource_negative','ore=8','ore=-1','resource amount'),
 ('building_ref','mill=2','wrong=2','unknown building'),
 ('building_negative','mill=2','mill=-1','building level'),
 ('infrastructure_negative','infrastructure=3','infrastructure=-1','infrastructure'),
 ('state_name','name_key="south"','name_key=""','state name_key'),
 ('vp_unknown','province=65535','province=999','unknown province'),
 ('vp_duplicate','province=65535','province=0','duplicate victory point'),
 ('vp_zero','points=2','points=0','positive'),
 ('vp_negative','points=2','points=-1','positive'),
 ('vp_name','name_key="lake_vp"','name_key=""','victory point name_key'),
 ('toml_unknown','infrastructure=3','wrong=3','unknown field')]:
  make(name,states=STATES.replace(old,new),err=('states.toml',msg,None))
make('truly_missing_land',states=STATES[STATES.index('[[state]]',1):],err=('states.toml','exactly one state',None))
make('syntax',states='state=[\n',err=('states.toml','unclosed array',None))
for name,text,msg in [('registry_duplicate','[[terrain]]\nid="plain"\n[[terrain]]\nid="plain"\n','duplicate registry'),('registry_type','[[terrain]]\nid=42\n','string'),('registry_id','[[terrain]]\nid="UPPER"\n','invalid'),('registry_missing','[[terrain]]\nname="plain"\n','missing registry')]:
 p=make(name,err=('terrain.toml',msg,None));(p/'common/terrain.toml').write_text(text)
p=make('missing_csv',err=('provinces.csv','',None)); (p/'maps/probe/provinces.csv').unlink()
for s in ['0','-1','nan','inf','2147483648','"1.25"','1e-20']:
 make('scale_'+str(len(CASES)),scale=s,err=('regions.toml','km_per_pixel',None))
make('distance_overflow',scale='2147483647',err=('regions.toml','distance overflow',None))
make('small_scale',scale='0.5')
make('exact_one_bit_scale',scale='0.00000000023283064365386962890625')
for kind,depth,label in [(0,8,'gray'),(6,8,'rgba'),(3,8,'palette'),(2,16,'rgb16')]:
 p=make(label,err=('provinces.png','24-bit RGB',None)); (p/'maps/probe/provinces.png').write_bytes(png(7,5,GRID,depth=depth,kind=kind))
p=make('apng',err=('provinces.png','24-bit RGB',None));(p/'maps/probe/provinces.png').write_bytes(png(7,5,GRID,apng=True))
p=make('png_truncated',err=('provinces.png','invalid PNG',None));q=p/'maps/probe/provinces.png';q.write_bytes(q.read_bytes()[:-5])
p=make('png_corrupt',err=('provinces.png','invalid PNG',None));q=p/'maps/probe/provinces.png';b=bytearray(q.read_bytes());b[-1]^=1;q.write_bytes(b)
p=make('png_zero_width',err=('provinces.png','invalid PNG',None));(p/'maps/probe/provinces.png').write_bytes(png(0,5,[]))
for text,msg in [('0,0,strait','self edge'),('0,999,strait','unknown province'),('0,7,river_small\n7,0,river_large','duplicate override'),('0,65535,impassable','requires bitmap'),('0,7,magic','invalid override')]:
 make('override_'+str(len(CASES)),over='a,b,kind\n'+text+'\n',err=('adjacency_overrides.csv',msg,None))
# 257 definitions force dense index 256 -> LE [0,1], independently of sparse IDs.
ids=[i*2 for i in range(257)]; colors=[[i&255,(i>>8)&255,51] for i in range(257)];grid=[i for i in ids for _ in range(2)]
csv='id,r,g,b,kind,terrain,coastal,island\n'+'\n'.join(f'{i},{c[0]},{c[1]},{c[2]},sea,water,false,false' for i,c in reversed(list(zip(ids,colors))))+'\n'
make('dense257',w=514,h=1,grid=grid,csv=csv,states='state=[]\n',colors=colors,ids=ids)
# Vertical height one / width one boundaries; no row-wrap or diagonal edges.
make('vertical',w=1,h=8,grid=[0,0,7,7,257,257,65535,65535])
make('diagonal_only',w=4,h=4,grid=[0,0,7,7,0,0,7,7,257,257,65535,65535,257,257,65535,65535])
# Deterministic random independent pixels, marked islands to focus on geometry/adjacency.
rng=random.Random(70491)
for i in range(20):
 g=IDS*2+[rng.choice(IDS) for _ in range(72)];rng.shuffle(g)
 make(f'random_{i:02}',w=10,h=8,grid=g,csv=CSV.replace(',false\n',',true\n'),warnings=[])
exe=BASE/'probe/target/debug/wp07_independent_probe.exe'
r=subprocess.run([str(exe)],input='\n'.join(c['path'] for c in CASES)+'\n',capture_output=True,text=True,encoding="utf-8")
(BASE/'probe-stderr.txt').write_text(r.stderr);assert r.returncode==0,r.stderr
outs=[json.loads(s) for s in r.stdout.splitlines()];assert len(outs)==len(CASES)
(BASE/'probe-results.json').write_text(json.dumps(list(zip([c['name'] for c in CASES],outs)),indent=2))
fail=[]
for c,out in zip(CASES,outs):
 try:
  if c['err']:
   file,msg,loc=c['err'];assert not out['ok'],out;assert out['file']==file,out;assert msg in out['message'],out
   if loc:assert [out['line'],out['column']]==list(loc),out
   assert out['line']>=1 and out['column']>=1
  else:
   assert out['ok'],out
   ids=sorted(c['ids']);g=c['grid'];w=c['w'];h=c['h'];idx=[ids.index(i) for i in g]
   assert (out['width'],out['height'])==(w,h);assert out['index']==idx
   assert out['bytes']==list(b''.join(struct.pack('<H',i) for i in idx));assert [p[0] for p in out['provinces']]==ids
   q=1<<32; centers=[]
   for id in ids:
    pts=[(i%w,i//w) for i,x in enumerate(g) if x==id]
    centers.append([sum(p[a] for p in pts)*q//len(pts)+q//2 for a in [0,1]])
   assert out['centers']==centers,(out['centers'],centers)
   # Lexemes chosen as exact binary rationals, not product parse-number.
   scales={'1.25':5*q//4,'0.5':q//2,'0.00000000023283064365386962890625':1};sc=scales[c['scale']];assert out['scale']==sc
   edges={}
   for y in range(h):
    for x in range(w):
     a=g[y*w+x]
     for xx,yy in [(x-1,y),(x,y-1),(x+1,y),(x,y+1)]:
      if 0<=xx<w and 0<=yy<h:
       b=g[yy*w+xx]
       if a!=b:edges[tuple(sorted([a,b]))]='Normal'
   for row in c['over'].splitlines()[1:]:
    a,b,k=row.split(',');edges[tuple(sorted([int(a),int(b)]))]={'river_small':'RiverSmall','river_large':'RiverLarge','strait':'Strait','impassable':'Impassable'}[k]
   expected=[]
   for (a,b),kind in sorted(edges.items()):
    ca=centers[ids.index(a)];cb=centers[ids.index(b)];n=sum((v-u)**2 for u,v in zip(ca,cb));assert n<2**127
    root=math.isqrt(n);assert root*root<=n<(root+1)*(root+1)
    expected.append([a,b,kind,(root*sc)//q])
   assert out['edges']==expected;assert out['reverse_edges'];assert out['unknown_index'] is None
   if c['warnings'] is not None:assert [x[0] for x in out['warnings']]==c['warnings']
   if c['name']!='dense257':
    assert out['owners']==[[0,0],[7,9],[257,None],[65535,None]]
    assert out['vp']==[[0,1,'land_vp'],[257,4,'sea_vp'],[65535,2,'lake_vp']]
    assert out['states']==[[0,'north',[0],123,{'ore':0},{'mill':0},0],[9,'south',[7],456,{'ore':8},{'mill':2},3]]
  print('PASS',c['name'])
 except Exception as e:
  fail.append((c['name'],str(e)));print('FAIL',c['name'],str(e))
print(f'{len(CASES)-len(fail)}/{len(CASES)} independent cases passed')
(BASE/'independent-summary.json').write_text(json.dumps({'total':len(CASES),'pass':len(CASES)-len(fail),'fail':fail},indent=2))
assert not fail,fail
