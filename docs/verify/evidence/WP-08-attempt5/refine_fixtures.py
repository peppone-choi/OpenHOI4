import run_checks as m,shutil
o=m.OUT
shutil.copy2(o/'lifecycle-results.json',o/'lifecycle-reload-fixture-results.json')
p=o/'lifecycle.cjs';t=p.read_text();t=t.replace("page.getByRole('alert')).toContainText('map')","page.getByRole('alert').filter({hasText:'Map data'})).toContainText('Map data')");p.write_text(t)
p=o/'ready-probe.cjs';t=p.read_text().replace("status=()=>page.getByRole('status').count()","status=()=>map.locator('..').getByRole('status').count()").replace("document.querySelector('[role=status]')","e.parentElement.querySelector('[role=status]')");p.write_text(t)
p=o/'ready_checks.py';t=p.read_text().replace("red62-ready'","red62-ready2'").replace("current-ready'","current-ready2'");p.write_text(t)
