import run_checks as m,struct,zlib,csv,urllib.request,json,hashlib
p=m.ROOT/'data/packs/testland/maps/testland/provinces.png';b=p.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n';pos=8;data=b''
while pos<len(b):
 n=struct.unpack('>I',b[pos:pos+4])[0];kind=b[pos+4:pos+8];part=b[pos+8:pos+8+n];assert zlib.crc32(kind+part)&0xffffffff==struct.unpack('>I',b[pos+8+n:pos+12+n])[0]
 if kind==b'IHDR':w,h,depth,ctype,compression,filtering,interlace=struct.unpack('>IIBBBBB',part)
 if kind==b'IDAT':data+=part
 pos+=12+n
assert (depth,ctype,compression,filtering,interlace)==(8,2,0,0,0)
raw=zlib.decompress(data);pixels=[];previous=[0]*(w*3)
def paeth(a,b,c):
 p=a+b-c; ds=[abs(p-a),abs(p-b),abs(p-c)];return [a,b,c][ds.index(min(ds))]
for y in range(h):
 f=raw[y*(1+w*3)];line=list(raw[y*(1+w*3)+1:(y+1)*(1+w*3)])
 for x in range(len(line)):
  a=line[x-3] if x>=3 else 0;bb=previous[x];c=previous[x-3] if x>=3 else 0
  line[x]=(line[x]+[0,a,bb,(a+bb)//2,paeth(a,bb,c)][f])&255
 pixels.extend(tuple(line[x:x+3]) for x in range(0,len(line),3));previous=line
rows=list(csv.DictReader((m.ROOT/'data/packs/testland/maps/testland/provinces.csv').open()));ids=sorted(int(r['id']) for r in rows);rgbToId={tuple(int(r[k]) for k in ['r','g','b']):int(r['id']) for r in rows};sourceIds=[rgbToId[c] for c in pixels]
base='http://127.0.0.1:19443'
with urllib.request.urlopen(base+'/maps/testland/metadata') as r:meta=json.load(r)
with urllib.request.urlopen(base+'/maps/testland/index.bin?pack='+meta['pack_hash']) as r:index=r.read();headers=dict(r.headers)
dense=list(struct.unpack('<'+'H'*(len(index)//2),index));actual=[meta['province_ids'][d] for d in dense];assert sourceIds==actual;assert (w,h)==(meta['width'],meta['height'])==(8,6);assert ids==meta['province_ids']==[10,20,30,40,50,60];assert len(index)==96
fnv=0xcbf29ce484222325
for v in index:fnv=((fnv^v)*0x100000001b3)&0xffffffffffffffff
assert f'{fnv:016x}'==meta['index_hash']=='09eff2825e821465';assert meta['pack_hash']=='b8d30ba577f303c3'
r={'time':m.now(),'pngSHA':hashlib.sha256(b).hexdigest(),'csvSHA':hashlib.sha256((m.ROOT/'data/packs/testland/maps/testland/provinces.csv').read_bytes()).hexdigest(),'decoder':'independent Python stdlib CRC/zlib/filter reconstruction + CSV RGB to stable ID','sourceIds':sourceIds,'httpDense':dense,'httpMappedIds':actual,'headers':headers,'meta':meta,'all48TexelsEqual':True}
(m.OUT/'independent-map-data.json').write_text(json.dumps(r,indent=2));print('Independent original PNG/CSV vs actual HTTP: all 48 texels PASS')
