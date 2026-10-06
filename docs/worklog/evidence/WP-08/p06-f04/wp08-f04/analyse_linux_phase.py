from pathlib import Path
import json,zipfile,sys,hashlib,math
root=Path(sys.argv[1]).resolve()
workspace=Path.cwd().resolve()
assert root.is_relative_to(workspace/'target')
assert root.is_dir() and (root.parent/'identity.json').is_file(), 'Validated ZIP extraction must finish first'
summary={'input':str(root),'geometry':[],'resize_traces':[],'test_reports':[]}
for p in sorted(root.rglob('*geometry.json')):
    j=json.loads(p.read_text(encoding='utf-8-sig'));samples=j['samples']
    if not isinstance(samples,list):continue
    groups=[];last=None;changes=[];pause=None
    for s in samples:
        c=s['canvas'];b=s['box'];key=(c['id'] if c else None,b['x'],b['y'],b['width'],b['height'],c['width'] if c else None,c['height'] if c else None)
        if key!=last:
            groups.append({'firstMs':s['ms'],'lastMs':s['ms'],'geometry':key,'mode':s['dataset'].get('mode'),'frames':s['dataset'].get('frames'),'tick':s['tick'],'pause':s['pause'],'loading':s['loading']});last=key
        else:groups[-1]['lastMs']=s['ms']
        if s['pause']!=pause:changes.append({'ms':s['ms'],'pause':s['pause'],'tick':s['tick']});pause=s['pause']
    ready=[]
    for s in samples:
        c=s['canvas'];b=s['box'];camera=json.loads(s['dataset']['camera']) if s['dataset'].get('camera') else None
        if c and camera and s['dataset'].get('frames') and 'Loading map' not in s['loading'] and c['width']==math.floor(b['width']*s['viewport']['dpr']) and c['height']==math.floor(b['height']*s['viewport']['dpr']) and math.isclose(camera['width']/camera['height'],b['width']/b['height'],rel_tol=1e-12):ready.append(s)
    summary['geometry'].append({'file':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'status':j['status'],'timeout':j['timeout'],'nodeElapsedMs':j['ended']-j['started'],'samples':len(samples),'canvasIds':sorted({s['canvas']['id'] for s in samples if s['canvas']}),'groups':groups,'pauseTransitions':changes,'errors':j['errors'],'readySamples':len(ready),'lastReadySample':ready[-1] if ready else None,'readyScope':'Published frame/camera + no loading + matching DPR buffer and camera/host aspect. Palette correctness only from separate actual PNG/readPixels assertions.'})
for p in sorted(root.rglob('report.json')):
    j=json.loads(p.read_text(encoding='utf-8-sig'));tests=[]
    def scan(suite):
        for spec in suite.get('specs',[]):
            for test in spec.get('tests',[]):
                for result in test.get('results',[]):tests.append({'file':spec.get('file'),'title':spec['title'],'project':test.get('projectName'),'duration':result.get('duration'),'startTime':result.get('startTime'),'status':result['status'],'error':result.get('error')})
        for child in suite.get('suites',[]):scan(child)
    for suite in j['suites']:scan(suite)
    summary['test_reports'].append({'file':p.relative_to(root).as_posix(),'resize':[t for t in tests if 'resize' in (t['file'] or '')],'overlaps':tests})
for p in sorted(root.rglob('trace.zip')):
    if 'map-resize' not in p.as_posix():continue
    calls=[]
    with zipfile.ZipFile(p) as z:
        for name in z.namelist():
            if not name.endswith('.trace'):continue
            before={}
            for line in z.read(name).splitlines():
                e=json.loads(line)
                if e.get('type')=='before':before[e['callId']]=e
                elif e.get('type')=='after' and e['callId'] in before:
                    b=before[e['callId']];calls.append({'traceStream':name,'class':b.get('class'),'api':b.get('apiName',b.get('method')),'title':b.get('title'),'parentId':b.get('parentId'),'startMs':b.get('startTime'),'durationMs':e['endTime']-b['startTime'],'wallTime':b.get('wallTime'),'params':b.get('params'),'error':e.get('error')})
    summary['resize_traces'].append({'file':p.relative_to(root).as_posix(),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'calls':calls,'stageCalls':[c for c in calls if c['traceStream']=='test.trace' and c['title'] in ['Screenshot','Set viewport size','After Hooks']]})
assert len(summary['geometry'])==4 and len(summary['resize_traces'])==4 and len(summary['test_reports'])==2, 'Expected full/isolated Chromium/WebKit evidence'
dest=workspace/'target/wp08-f04/linux-phase/phase-summary.json'
dest.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
for j in summary['geometry']:print(json.dumps({k:j[k] for k in ['file','status','nodeElapsedMs','samples','canvasIds','groups','pauseTransitions']},ensure_ascii=False))
for j in summary['resize_traces']:
    print(j['file']);print(json.dumps(sorted(j['calls'],key=lambda c:c['durationMs'],reverse=True)[:12],ensure_ascii=False))
