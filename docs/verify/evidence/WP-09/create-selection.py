from pathlib import Path
r=Path(r'E:/openhoi/.orchestrator/wt/WP-09-verify/target/wp09-verify')
t=(r/'browser-probes.mjs').read_text(encoding='utf8')
a=t.index('const cases=[');b=t.index(';const products=',a)
t=t[:a]+'''const cases=[['sparse-selection',m=>{const nationId=id=>id===1?40001:id===2?60002:id;const stateId=id=>id===1?30001:id===2?50002:id;m.world.nations.forEach(n=>n.id=nationId(n.id));m.world.states.forEach(s=>{s.id=stateId(s.id);s.owner=nationId(s.owner)});m.world.provinces.forEach(p=>{p.state=stateId(p.state);p.owner=nationId(p.owner);p.controller=nationId(p.controller)})},'selection']]'''+t[b:]
t=t.replace("out=root+'/target/wp09-verify/adversarial'","out=root+'/target/wp09-verify/selection'")
t=t.replace("if(kind==='precision')", "if(kind==='selection'){const country=page.getByTestId('country-panel'),state=page.getByTestId('state-panel');await country.getByRole('button',{name:'Southern Test Nation'}).click();assert.ok((await country.textContent()).includes('STH'));assert.ok((await country.textContent()).includes('Test Council'));await state.getByRole('button',{name:'Southern Test State'}).click();assert.ok((await state.textContent()).includes('2000'));assert.equal(await state.locator('.ledger-value strong').textContent(),'0');await state.getByRole('button',{name:'Northern Test State'}).click();assert.equal(await page.getByTestId('province-20').textContent(),'20Northern Test NationSouthern Test Nation');row.sparseIds={nations:[40001,60002],states:[30001,50002]};}if(kind==='precision')")
(r/'selection-probes.mjs').write_text(t,encoding='utf8')

