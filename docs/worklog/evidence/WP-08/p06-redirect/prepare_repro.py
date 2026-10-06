from pathlib import Path
root=Path(r'E:/openhoi/.orchestrator/wt/WP-08/target/wp08/p06-redirect')
for resource,filename,output in [('metadata','independent-repro-original.cjs','cors-valid-redirect'),('index','independent-index-repro-original.cjs','cors-valid-index-redirect')]:
    original=(root/filename).read_text(encoding='utf-8')
    for label,port in [('red',19420),('green',19422)]:
        source=original.replace("require('../../client/node_modules/@playwright/test')","require('../../../client/node_modules/@playwright/test')")
        source=source.replace('19412',str(port)).replace(f"__dirname+'/{output}",f"__dirname+'/{label}-independent-{resource}-repro")
        source=source.replace("commit:'f58c0c61322287f48318bce47ca51deb7daebcc9'",f"implementation:'{label} WP08 P06; original independent fixture f58c0c6'")
        (root/f'{label}-independent-{resource}-repro.cjs').write_text(source,encoding='utf-8')
