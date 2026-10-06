import run_checks as m
from own_runtime import Runtime
with Runtime(m.ROOT,19443,'extra-current'):
 for i in (1,2):
  out=m.OUT/f'current-repeat{i}';out.mkdir(exist_ok=True);(out/'resize-probe.cjs').write_bytes((m.OUT/'current-original-probes/resize-probe.cjs').read_bytes());m.run(f'current-repeat{i}',['node',str(out/'resize-probe.cjs')])
 m.run('independent-lifecycle',['node',str(m.OUT/'lifecycle.cjs')])
