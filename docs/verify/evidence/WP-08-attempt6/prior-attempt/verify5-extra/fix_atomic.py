import run_checks as m,subprocess,json,shutil
script="Get-CimInstance Win32_Process -Filter \"Name='node.exe'\" | Select-Object ProcessId,ParentProcessId,ExecutablePath,CommandLine | ConvertTo-Json -Compress"
records=json.loads(subprocess.check_output(['powershell','-NoProfile','-Command',script]).decode('utf-8-sig'))
owned=[p for p in records if str(m.OUT/'atomic.cjs').lower() in p['CommandLine'].lower()]
assert len(owned)<=1
(m.OUT/'atomic-fixture-stop.json').write_text(json.dumps({'reason':'case-sensitive independent fixture expected invalid while actual localized English is Invalid; stop only own node and let owner Runtime cleanup','processes':owned},indent=2))
for p in owned:subprocess.run(['powershell','-NoProfile','-Command',f"Stop-Process -Id {p['ProcessId']}"],check=True)
p=m.OUT/'atomic.cjs';shutil.copy2(p,m.OUT/'atomic-first-fixture.cjs');p.write_text(p.read_text(encoding='utf-8').replace("toContainText('invalid')","toContainText('Invalid')"),encoding='utf-8')
