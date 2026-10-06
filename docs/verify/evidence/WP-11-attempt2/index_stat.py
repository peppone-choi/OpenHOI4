import struct,pathlib,json,datetime,hashlib
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';b=pathlib.Path('E:/openhoi/.orchestrator/evidence/WP11.verify2.before.index').read_bytes();sig,ver,count=struct.unpack('>4sII',b[:12]);assert sig==b'DIRC' and ver in [2,3];pos=12;changed=[]
for n in range(count):
 start=pos;entry=struct.unpack('>10I20sH',b[pos:pos+62]);pos+=62
 if ver==3 and entry[-1]&0x4000:pos+=2
 end=b.index(b'\0',pos);name=b[pos:end].decode();pos=end+1;pos=start+((pos-start+7)//8)*8
 stat=(r/name).stat();stored_ns=entry[2]*1000000000+entry[3]
 if abs(stored_ns-stat.st_mtime_ns)>100:
  changed.append({'path':name,'stored_index_mtime_utc':datetime.datetime.fromtimestamp(stored_ns/1e9,datetime.timezone.utc).isoformat(),'current_mtime_utc':datetime.datetime.fromtimestamp(stat.st_mtime_ns/1e9,datetime.timezone.utc).isoformat(),'index_size':entry[9],'current_size':stat.st_size})
(o/'tracked-stat-differences.json').write_text(json.dumps({'index_version':ver,'entry_count':count,'stat_differences':changed},indent=2));print(json.dumps({'version':ver,'stat_differences':len(changed),'paths':[x['path'] for x in changed]}))
