import pathlib,subprocess,json,re
root=pathlib.Path.cwd();ev=root/'target/evidence/M0-gate-verify';spec=(root/'docs/02-production-spec.md').read_text(encoding='utf8');design=(root/'docs/01-game-design.md').read_text(encoding='utf8');rows=[l for l in spec.splitlines() if re.match(r'\| REQ-',l) and '| M0 | 필수 |' in l]
result={'m0_required_rows':rows,'ancestors':{},'report_integrities':{},'product_diff':subprocess.run(['git','diff','--exit-code','f12b30345c7255bdceb4d0e6b32405f5c48dd5b6','HEAD','--','crates','client','data','assets','tools','.github','Cargo.toml','Cargo.lock','deny.toml','rust-toolchain.toml','AGENTS.md'],capture_output=True,text=True).returncode}
assert len(rows)==14 and result['product_diff']==0
for wp,sha in [('01','5f5da6273d7177f655ebaaaf4ef763b741920c7d'),('02','eb15633543a67eb658a4d5d4b67e33fe7ce07ea6'),('03','70de3192b574bc7fd5ff1e97d4169bde9507ed71'),('04','88d730a'),('05','64789f5517556ed28fe19120f02bc7645887f09a'),('06','1ad982b5a50f887c48e5c51c2af771d91903c3e0')]:
 p=subprocess.run(['git','merge-base','--is-ancestor',sha,'f12b30345c7255bdceb4d0e6b32405f5c48dd5b6']);assert p.returncode==0,(wp,sha);result['ancestors'][wp]=sha
 report=(root/f'docs/verify/WP-{wp}.md').read_text(encoding='utf8');assert '**PASS' in report
 files=list((root/f'docs/verify/evidence/WP-{wp}').glob('*integrity*.json'))
 result['report_integrities'][wp]=[{'path':str(p.relative_to(root)),'content':json.loads(p.read_text(encoding='utf-8-sig'))} for p in files]
for p in ['AGENTS.md','tools/orch.sh','tools/check_docs.py','docs/templates/WORKLOG_TEMPLATE.md','docs/templates/VERIFY_REPORT_TEMPLATE.md','docs/templates/HANDOFF_TEMPLATE.md','docs/clean-room-checklist.md']:
 assert (root/p).is_file(),p
original='fd4d167';baseline={}
for p in ['tools/orch.sh','tools/check_docs.py','AGENTS.md']+subprocess.check_output(['git','ls-files','docs/templates'],text=True).splitlines():
 try:
  a=subprocess.check_output(['git','show',original+':'+p],stderr=subprocess.DEVNULL);b=subprocess.check_output(['git','show','HEAD:'+p]);baseline[p]=a==b
 except subprocess.CalledProcessError:baseline[p]='not in baseline'
result['original_content_comparison']=baseline
(ev/'scope-and-integration-audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf8');print('M0 rows',len(rows));print('integrated ancestors',result['ancestors']);print('original content',baseline)
for r in rows:print(r)
