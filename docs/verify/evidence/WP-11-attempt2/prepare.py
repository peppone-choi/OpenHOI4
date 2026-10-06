import pathlib,json,shutil
r=pathlib.Path.cwd(); o=r/'target/wp11-verify2'; h=o/'probe';(h/'src').mkdir(parents=True,exist_ok=True)
manifest='[package]\nname="wp11_verify2_probe"\nversion="0.0.0"\nedition="2024"\n[workspace]\n[dependencies]\n'
for name in ['oh_core','oh_data','oh_sim','oh_save','oh_proto']:
 manifest+=f'{name}={{path="{(r/"crates"/name).as_posix()}"}}\n'
manifest+='serde_json="=1.0.151"\nzstd={version="=0.13.3",default-features=false,features=["no_asm"]}\n'
(h/'Cargo.toml').write_text(manifest)
p=o/'own-pack';shutil.copytree(r/'data/packs/testland',p,dirs_exist_ok=True)
s=p/'scenarios/m1/scenario.toml';text=s.read_text().replace('2000-01-01','2000-02-28')
text+="\n[state_modifiers]\n1=[{source='probe.bit',target_stat='infrastructure',operation='add',value='0.00000000023283064365386962890625',expires=24},{source='probe.mul',target_stat='infrastructure',operation='mul',value='1.5'}]\n"
s.write_text(text,newline='\n')
