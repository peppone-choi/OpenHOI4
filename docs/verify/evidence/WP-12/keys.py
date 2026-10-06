import pathlib,re,json
r=pathlib.Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');keys={lang:set(re.findall(r'^([a-z][a-z0-9-]+) =', (r/f'client/public/locales/{lang}.ftl').read_text(encoding='utf-8'),re.M)) for lang in ['ko','en']}
sources=['client/src/App.tsx','client/src/components/GameShell.tsx','client/src/components/TimeControls.tsx','client/src/components/LedgerValue.tsx']
static={k for p in sources for k in re.findall(r"t\('([a-z][a-z0-9-]+)'",(r/p).read_text(encoding='utf-8'))}
server={k for p in (r/'crates/oh_server/src').glob('*.rs') for k in re.findall(r'"([a-z][a-z0-9]+(?:-[a-z0-9]+)+)"',p.read_text(encoding='utf-8')) if k in {'protocol-version','hello-required','invalid-message','invalid-speed','invalid-sequence','not-joined','unsupported-session','unsupported-create','already-joined','unsupported-query','simulation-error','session-closed'}}
result={'static_ui_keys':sorted(static),'server_reason_keys':sorted(server),'missing':{l:sorted((static|server)-ks) for l,ks in keys.items()}}
print(json.dumps(result,indent=2));(r/'target/wp12-verify2/key-coverage.json').write_text(json.dumps(result,indent=2));assert not any(result['missing'].values())
