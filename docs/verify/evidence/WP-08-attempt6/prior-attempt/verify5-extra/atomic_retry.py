import run_checks as m,shutil
from own_runtime import Runtime
for f in ['atomic-final.log','atomic-results.json']:shutil.copy2(m.OUT/f,m.OUT/('pending-query-fixture-'+f))
p=m.OUT/'atomic.cjs';t=p.read_text(encoding='utf-8').replace('let inject=false,done=false;','let inject=false,done=false,armed=false;').replace('sent.push(decode(data));server.send(data);',"const value=decode(data);sent.push(value);if(armed&&value.type==='Command')inject=true;server.send(data);").replace('const before=await ui();inject=true;', 'const before=await ui();armed=true;');p.write_text(t,encoding='utf-8')
with Runtime(m.ROOT,19443,'atomic-armed'):m.run('atomic-armed',['node',str(p)])
