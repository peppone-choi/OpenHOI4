import struct,pathlib,json,hashlib
r=pathlib.Path.cwd();o=r/'target/wp11-verify2'
def parse(path):
 b=path.read_bytes();v,n=struct.unpack('>II',b[4:12]);pos=12;items={}
 for _ in range(n):
  st=pos;ent=struct.unpack('>10I20sH',b[pos:pos+62]);pos+=62
  if v==3 and ent[-1]&0x4000:pos+=2
  e=b.index(b'\0',pos);name=b[pos:e].decode();pos=e+1;pos=st+((pos-st+7)//8)*8;items[name]={'stat':list(ent[:10]),'oid':ent[10].hex(),'flags':ent[-1]}
 return b,items,b[pos:-20]
b,before,ext1=parse(o/'final.index');a,after,ext2=parse(o/'final-confirm.index');changes=[{'path':k,'before':before[k],'after':v} for k,v in after.items() if before[k]!=v]
record={'first_unchanged_snapshot':'final.json at 2026-10-07 07:07 KST','first_changed_snapshot':'final-confirm.json after forensic diff/stat and index_stat.py','before_sha':hashlib.sha256(b).hexdigest(),'after_sha':hashlib.sha256(a).hexdigest(),'entry_changes':changes,'extension_before_hex':ext1.hex(),'extension_after_hex':ext2.hex(),'current_source_attribution':'unresolved; all this session shell environments set GIT_OPTIONAL_LOCKS=0; no restoration/config/index flag changes performed'};(o/'raw-index-first-change.json').write_text(json.dumps(record,indent=2));print(json.dumps({'before_sha':record['before_sha'],'after_sha':record['after_sha'],'entry_changes':len(changes),'entries':[{'path':x['path'],'before':x['before'],'after':x['after']} for x in changes[:12]],'extensions_equal':ext1==ext2}))
