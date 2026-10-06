import run_checks as m,time
from own_runtime import Runtime
with Runtime(m.ROOT,19444,'gui') as r:
 print('GUI runtime ready ownPID='+str(r.p.pid),flush=True)
 while not (m.OUT/'gui-stop').exists():time.sleep(.2)
