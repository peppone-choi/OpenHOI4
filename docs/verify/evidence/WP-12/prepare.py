from pathlib import Path
r=Path('E:/openhoi/.orchestrator/wt/WP-12-verify2');o=r/'target/wp12-verify2';prev=Path('E:/openhoi/.orchestrator/wt/WP-12-verify/target/wp12-verify')
for name in ['audit.py','glyph.py','keys.py','proto_check.py','playwright.config.ts','e2e/independent.spec.ts']:
 p=o/name;p.parent.mkdir(parents=True,exist_ok=True)
 s=(prev/name).read_text(encoding='utf-8').replace('WP-12-verify/','WP-12-verify2/').replace('WP-12-verify\'','WP-12-verify2\'').replace('wp12-verify','wp12-verify2')
 p.write_text(s,encoding='utf-8')
