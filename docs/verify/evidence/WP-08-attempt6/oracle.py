import pathlib,struct,zlib,csv,json,urllib.request,hashlib
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6'
def decode(path):
 b=path.read_bytes();assert b[:8]==b'\x89PNG\r\n\x1a\n';p=8;ids=[]
 while p<len(b):
  n=struct.unpack('>I',b[p:p+4])[0];kind=b[p+4:p+8];data=b[p+8:p+8+n]
  assert zlib.crc32(kind+data)&0xffffffff==struct.unpack('>I',b[p+8+n:p+12+n])[0]
  if kind==b'IHDR':w,h,depth,color,compression,filtering,interlace=struct.unpack('>IIBBBBB',data);assert depth==8 and color in (2,6) and interlace==0
  if kind==b'IDAT':ids.append(data)
  p+=n+12
 c=3 if color==2 else 4;stride=w*c;raw=zlib.decompress(b''.join(ids));rows=[];prior=[0]*stride
 for y in range(h):
  f=raw[y*(stride+1)];line=list(raw[y*(stride+1)+1:(y+1)*(stride+1)])
  for x in range(stride):
   a=line[x-c] if x>=c else 0;u=prior[x];ul=prior[x-c] if x>=c else 0
   if f==1:predict=a
   elif f==2:predict=u
   elif f==3:predict=(a+u)//2
   elif f==4:
    value=a+u-ul;dist=[abs(value-a),abs(value-u),abs(value-ul)];predict=[a,u,ul][dist.index(min(dist))]
   else:assert f==0;predict=0
   line[x]=(line[x]+predict)%256
  rows.append([line[x:x+3] for x in range(0,stride,c)]);prior=line
 return w,h,rows
if __name__=='__main__':
 # HTTP exclusively from already independently identified own server PID19453.
 meta=json.load(urllib.request.urlopen('http://127.0.0.1:19453/maps/testland/metadata'));req=urllib.request.urlopen('http://127.0.0.1:19453/maps/testland/index.bin?pack='+meta['pack_hash']);raw=req.read()
 src=ROOT/'data/packs/testland/maps/testland';w,h,pixels=decode(src/'provinces.png');rows=list(csv.DictReader((src/'provinces.csv').open(encoding='utf-8-sig')));colors={tuple(int(r[k]) for k in ['r','g','b']):int(r['id']) for r in rows};ids=sorted(colors.values());expected=[ids.index(colors[tuple(pixel)]) for row in pixels for pixel in row];actual=list(struct.unpack('<'+'H'*(len(raw)//2),raw));assert expected==actual
 hash=0xcbf29ce484222325
 for byte in raw:hash=((hash^byte)*0x100000001b3)&0xffffffffffffffff
 assert [w,h]==[meta['width'],meta['height']] and ids==meta['province_ids'] and len(raw)==int(meta['byte_length']) and f'{hash:016x}'==meta['index_hash']
 result={'metadata':meta,'headers':dict(req.headers),'indexSHA256':hashlib.sha256(raw).hexdigest(),'all48texels_equal':True,'sourceCSV_sortedIDs':ids,'sourcePNG_sha256':hashlib.sha256((src/'provinces.png').read_bytes()).hexdigest(),'sourceCSV_sha256':hashlib.sha256((src/'provinces.csv').read_bytes()).hexdigest(),'actual':actual,'expected':expected,'independent_decoder':'Python stdlib struct/zlib CRC+PNG filter decode, CSV RGB map -> sorted dense, no product model import'}
 (OUT/'index-oracle.json').write_text(json.dumps(result,indent=2),encoding='utf8');(OUT/'index.bin').write_bytes(raw);print('PASS all48texels',f'{hash:016x}',meta['pack_hash'])
