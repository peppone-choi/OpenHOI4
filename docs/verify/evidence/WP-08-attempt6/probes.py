import pathlib,sys,json
import run as runner
from runtime import Runtime,ROOT,OUT
mode=sys.argv[1]
if mode=='redgreen':
 for label,root,port in [('red62',OUT/'red62-source',19455),('red953',OUT/'red953-source',19456),('current',ROOT,19453)]:
  dest=OUT/(label+'-probes');names=['resize-probe.cjs','gpu-compile-failure.cjs','ready-probe.cjs'] if label=='red62' else ['lifecycle.cjs'] if label=='red953' else ['resize-probe.cjs','gpu-compile-failure.cjs','ready-probe.cjs','initialization.cjs','lifecycle.cjs','live-pipeline.cjs','atomic.cjs']
  with Runtime(root,port,label+'-probe'):
   for name in names:runner.run(label+'-'+name,['node',str(dest/name)],ROOT,{'PROBE_OUT':str(dest)})
