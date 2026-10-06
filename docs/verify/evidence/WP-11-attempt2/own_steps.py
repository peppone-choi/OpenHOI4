import json,pathlib
r=pathlib.Path.cwd();o=r/'target/wp11-verify2';probe=str(o/'probe-build/debug/wp11_verify2_probe.exe');pack=str(o/'own-pack');out=str(o/'own-native');cli=str(r/'target/debug/oh_cli.exe')
steps=[['proto',[probe,'proto',str(o/'generated/protocol.ts')]],['own-capture',[probe,'capture',pack,out]],['own-resume',[probe,'resume',pack,str(o/'own-native/saved.ohsave')]],['own-cli-resume',[cli,'resume','--load',str(o/'own-native/saved.ohsave'),'--pack',pack,'--ticks','37','--hash-out']],['invalid-queue',[probe,'invalid',pack]]]
for i in (1,2):
 steps.append([f'm0-hash{i}',[cli,'run','--scenario','testland','--days','365','--seed','1','--hash-out']]);steps.append([f'm1-hash{i}',[cli,'run','--pack','data/packs/testland','--scenario','m1','--ticks','1000','--seed','1','--hash-out']])
(o/'own-steps.json').write_text(json.dumps(steps))
