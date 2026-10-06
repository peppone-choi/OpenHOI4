import pathlib,urllib.request,urllib.error,json,csv,struct,zlib,hashlib,shutil,subprocess
from run import ROOT,OUT,run
base='http://127.0.0.1:19415';results={}
def response(path):
 try:
  with urllib.request.urlopen(base+path) as r:return r.status,dict(r.headers),r.read()
 except urllib.error.HTTPError as e:return e.code,dict(e.headers),e.read()
status,headers,body=response('/maps/testland/metadata');assert status==200 and headers['cache-control']=='no-cache';meta=json.loads(body)
status,ih,raw=response('/maps/testland/index.bin?pack='+meta['pack_hash']);assert status==200 and ih['x-pack-hash']==meta['pack_hash'] and 'immutable' in ih['cache-control']
assert meta['schema_version']==1 and meta['byte_length']==str(len(raw)) and len(raw)==meta['width']*meta['height']*2
fnv=0xcbf29ce484222325
for b in raw:fnv=((fnv^b)*0x100000001b3)&((1<<64)-1)
assert f'{fnv:016x}'==meta['index_hash']==ih['x-index-hash']
png=(ROOT/'data/packs/testland/maps/testland/provinces.png').read_bytes();offset=8;compressed=b''
while offset<len(png):
 size=struct.unpack('>I',png[offset:offset+4])[0];kind=png[offset+4:offset+8];chunk=png[offset+8:offset+8+size]
 if kind==b'IHDR':w,h,depth,color,compression,filter_,interlace=struct.unpack('>IIBBBBB',chunk)
 if kind==b'IDAT':compressed+=chunk
 offset+=size+12
assert (w,h,depth,color,compression,filter_,interlace)==(8,6,8,2,0,0,0)
data=zlib.decompress(compressed);stride=w*3;previous=[0]*stride;pixels=[]
def paeth(a,b,c):
 p=a+b-c;pa,pb,pc=abs(p-a),abs(p-b),abs(p-c)
 return a if pa<=pb and pa<=pc else b if pb<=pc else c
for y in range(h):
 kind=data[y*(stride+1)];row=list(data[y*(stride+1)+1:(y+1)*(stride+1)])
 for i in range(stride):
  a=row[i-3] if i>=3 else 0;b=previous[i];c=previous[i-3] if i>=3 else 0
  prediction=[0,a,b,(a+b)//2,paeth(a,b,c)][kind];row[i]=(row[i]+prediction)&255
 pixels.extend(tuple(row[i:i+3]) for i in range(0,stride,3));previous=row
with (ROOT/'data/packs/testland/maps/testland/provinces.csv').open() as f:rows=list(csv.DictReader(f))
palette={tuple(int(row[c]) for c in ['r','g','b']):int(row['id']) for row in rows};stableIds=sorted(palette.values());assert stableIds==meta['province_ids']
expected=[stableIds.index(palette[pixel]) for pixel in pixels];actual=list(struct.unpack('<'+'H'*(w*h),raw));assert actual==expected
results['indexOracle']={'pixels':w*h,'ids':stableIds,'rawSHA256':hashlib.sha256(raw).hexdigest(),'fnv':f'{fnv:016x}','method':'stdlib PNG RGB filters + independent CSV palette -> dense ID; byte-for-byte HTTP LEu16 comparison'}
cases=[('/maps/testland/index.bin',409),('/maps/testland/index.bin?pack=wrong',409),('/maps/testland/index.bin?pack=',409),('/maps/testland/index.bin?pack='+meta['pack_hash'].upper(),409),('/maps/unknown/metadata',404),('/maps/unknown/index.bin?pack='+meta['pack_hash'],404)]
results['http']={}
for path,expectedStatus in cases:
 status,_,_=response(path);assert status==expectedStatus,(path,status);results['http'][path]=status
for i in range(3):assert response('/maps/testland/index.bin?pack='+meta['pack_hash'])[2]==raw
results['repeatCacheIdentity']=True
missing=OUT/'missing-pack';missing.mkdir(exist_ok=True);destination=missing/'testland'
shutil.copytree(ROOT/'data/packs/testland',destination,ignore=shutil.ignore_patterns('provinces.png'),dirs_exist_ok=True)
exit=run('missing-png',[str(ROOT/'target/debug/oh_server.exe'),'--port','19417','--pack-root',str(missing)]);assert exit==1
(OUT/'http.json').write_text(json.dumps(results,indent=2),encoding='utf-8');print(json.dumps(results,indent=2))
