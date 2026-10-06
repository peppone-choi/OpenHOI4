from pathlib import Path
import subprocess
root=Path.cwd().resolve()
dist=(root/'client/dist').resolve(); saved=(root/'target/wp05/saved-dist').resolve()
assert dist.is_relative_to(root) and saved.is_relative_to(root) and not saved.exists()
dist.rename(saved)
try:
 result=subprocess.run(['cargo','check','-p','oh_server'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
 (root/'target/wp05/missing-build.log').write_bytes(result.stdout)
 assert result.returncode != 0 and b'built client missing' in result.stdout
 print('missing real client build: expected nonzero exit',result.returncode)
finally:
 saved.rename(dist)
result=subprocess.run(['cargo','check','-p','oh_server'],stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(root/'target/wp05/restored-build.log').write_bytes(result.stdout)
assert result.returncode == 0
print('restored real client: cargo check exit 0')
