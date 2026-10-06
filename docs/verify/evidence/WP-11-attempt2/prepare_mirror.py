import pathlib,shutil,tomllib,json,hashlib,subprocess
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';mirror=o/'mirror';crate=mirror/'crates/oh_save';shutil.copytree(r/'crates/oh_save',crate,dirs_exist_ok=True);shutil.copytree(r/'data/packs/testland',mirror/'data/packs/testland',dirs_exist_ok=True)
manifest='[package]\nname="oh_save"\nversion="0.1.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
orig=tomllib.loads((r/'crates/oh_save/Cargo.toml').read_text())
for n,v in orig['dependencies'].items():
 if v.get('workspace') if isinstance(v,dict) else False:manifest+=f'{n}={{path="{(r/"crates"/n).as_posix()}"}}\n'
 elif isinstance(v,str):manifest+=f'{n}="{v}"\n'
 else:manifest+=n+'={'+','.join(k+'='+json.dumps(val) for k,val in v.items())+'}\n'
manifest+='[dev-dependencies]\nserde_json="=1.0.151"\n';(crate/'Cargo.toml').write_text(manifest)
# Preserve exact Git source; only the ignored copy gets a new verifier-owned test.
source=subprocess.check_output(['git','--no-optional-locks','-C',str(r),'show','HEAD:crates/oh_save/src/file.rs']);assert source==(r/'crates/oh_save/src/file.rs').read_bytes();(o/'transaction-original-source.rs').write_bytes(source)
(o/'mirror-source.json').write_text(json.dumps({'head':'750c2732b00c164ddc252fa26c4984ababd64f35','original_file_sha':hashlib.sha256(source).hexdigest(),'original_bytes':len(source)}))
