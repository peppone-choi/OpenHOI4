import pathlib,json,csv,struct,zlib,urllib.request,hashlib,subprocess,time
r=pathlib.Path.cwd();o=r/'target/wp08-verify3';root=r/'data/packs/testland/maps/testland';base='http://127.0.0.1:19428'
with (o/'index-server.log').open('w') as log:
 p=subprocess.Popen([str(r/'target/debug/oh_server.exe'),'--port','19428','--pack-root',str(r/'data/packs')],cwd=r,stdout=log,stderr=subprocess.STDOUT)
 try:
  for _ in range(50):
   try:meta=json.load(urllib.request.urlopen(base+'/maps/testland/metadata'));break
   except urllib.error.URLError:time.sleep(.1)
  response=urllib.request.urlopen(base+'/maps/testland/index.bin?pack='+meta['pack_hash']);raw=response.read();(o/'actual-index.bin').write_bytes(raw)
  rows=sorted(csv.DictReader((root/'provinces.csv').open(encoding='utf-8')),key=lambda x:int(x['id']));ids=[int(row['id']) for row in rows];lookup={tuple(int(row[k]) for k in ['r','g','b']):i for i,row in enumerate(rows)};source=(root/'provinces.png').read_bytes();pos=8;chunks=[]
  while pos<len(source):
   n=struct.unpack('>I',source[pos:pos+4])[0];kind=source[pos+4:pos+8];data=source[pos+8:pos+8+n];pos+=12+n
   if kind==b'IHDR':w,h,depth,color,_,_,interlace=struct.unpack('>IIBBBBB',data)
   if kind==b'IDAT':chunks.append(data)
  assert depth==8 and color in (2,6) and interlace==0;channels=3 if color==2 else 4;stride=w*channels;decoded=zlib.decompress(b''.join(chunks));pixels=bytearray(h*stride)
  def paeth(a,b,c):
   v=a+b-c;pa,pb,pc=abs(v-a),abs(v-b),abs(v-c);return a if pa<=pb and pa<=pc else b if pb<=pc else c
  for y in range(h):
   f=decoded[y*(stride+1)]
   for x in range(stride):
    left=pixels[y*stride+x-channels] if x>=channels else 0;up=pixels[(y-1)*stride+x] if y else 0;corner=pixels[(y-1)*stride+x-channels] if y and x>=channels else 0
    predictor=[0,left,up,(left+up)//2,paeth(left,up,corner)][f];pixels[y*stride+x]=(decoded[y*(stride+1)+1+x]+predictor)&255
  expected=[lookup[tuple(pixels[i*channels:i*channels+3])] for i in range(w*h)];actual=list(struct.unpack('<'+'H'*(w*h),raw));assert expected==actual;assert ids==meta['province_ids'];assert (w,h)==(meta['width'],meta['height']);assert len(raw)==int(meta['byte_length'])==w*h*2
  fnv=0xcbf29ce484222325
  for b in raw:fnv=((fnv^b)*0x100000001b3)&0xffffffffffffffff
  assert f'{fnv:016x}'==meta['index_hash'];(o/'index-source-oracle.json').write_text(json.dumps({'meta':meta,'headers':dict(response.headers),'csv':rows,'pngSha256':hashlib.sha256(source).hexdigest(),'indexSha256':hashlib.sha256(raw).hexdigest(),'decodedPNGExpectedDense':expected,'actualHTTPDense':actual,'independentSourceOracle':True,'matchingFNV':f'{fnv:016x}'},indent=2));print('PNG/CSV/HTTP dense oracle PASS',w,h,ids,meta['pack_hash'],meta['index_hash'])
 finally:p.terminate();p.wait(timeout=10)
