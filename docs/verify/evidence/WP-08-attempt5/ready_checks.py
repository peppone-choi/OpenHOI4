import run_checks as m
from own_runtime import Runtime
for label,root in [('red62-ready2',m.OUT/'red62-source'),('current-ready2',m.ROOT)]:
 out=m.OUT/label;out.mkdir(exist_ok=True)
 with Runtime(root,19444,label):m.run(label,['node',str(m.OUT/'ready-probe.cjs')],extra={'PROBE_OUT':str(out)})
