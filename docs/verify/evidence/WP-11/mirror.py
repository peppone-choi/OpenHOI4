from run import *
import shutil,re
dest=OUT/'transaction-mirror';shutil.copytree(ROOT/'crates/oh_save',dest,dirs_exist_ok=True)
source=(ROOT/'crates/oh_save/src/file.rs').read_bytes();addition=(OUT/'transaction-probe.rs').read_bytes();(dest/'src/file.rs').write_bytes(source+b'\n'+addition)
manifest=(dest/'Cargo.toml').read_text().replace('name = "oh_save"','name = "oh_save_transaction_probe"')
for k,v in [('rust-version','"1.99"'),('version','"0.1.0"'),('edition','"2024"'),('publish','false')]:manifest=re.sub('^'+k+r'\.workspace = true$',k+' = '+v,manifest,flags=re.M)
manifest=manifest.replace('[lints]\nworkspace = true','[workspace]')
for c in ['oh_sim','oh_core','oh_data']:manifest=manifest.replace(c+'.workspace = true',c+' = { path = "../../../crates/'+c+'" }')
(dest/'Cargo.toml').write_text(manifest)
records=[]
for p in (ROOT/'crates/oh_save/src').glob('*.rs'):
 original=p.read_bytes();mirror=(dest/'src'/p.name).read_bytes();assert mirror.startswith(original) if p.name=='file.rs' else mirror==original
 records.append(dict(file=p.name,sha256=hashlib.sha256(original).hexdigest(),production_prefix_exact=True,added_test_bytes=len(addition) if p.name=='file.rs' else 0))
(OUT/'transaction-source-provenance.json').write_text(json.dumps(records,indent=2));print(json.dumps(records))
