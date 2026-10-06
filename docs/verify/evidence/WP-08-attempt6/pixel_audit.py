import pathlib,json,math,hashlib
from oracle import decode
ROOT=pathlib.Path(__file__).resolve().parents[2];OUT=ROOT/'target/wp08-verify6';records=[]
sets=[('Windows',OUT/'m1/map',['chrome','edge','chromium','firefox','webkit']),('LinuxCW',OUT/'independent-ci/wp05-protocol-browser-evidence/target/wp08/e2e',['chromium','webkit']),('LinuxFirefoxXvfb',OUT/'independent-ci/wp05-protocol-browser-evidence/target/wp08/ci/firefox-xvfb-map',['firefox'])]
baseline=decode(ROOT/'client/e2e-m1/baselines/display-fixture.png')
for platform,directory,browsers in sets:
 for browser in browsers:
  for force in ['false','true']:
   path=directory/(browser+'-borders-'+force+'.png');w,h,image=decode(path);assert [w,h]==list(baseline[:2]);viewHeight=max(6,12/(w/h))/.920;viewWidth=viewHeight*w/h;lineY=math.floor((1-3)/viewHeight*h+h/2)
   colors=[[100,116,135],[230,212,167],[16,24,32]];widths=[];sample=[]
   for x,color in zip([2,4,6],colors):
    px=math.floor((x-6)/viewWidth*w+w/2);actual=image[lineY][px];assert all(abs(a-b)<=1 for a,b in zip(actual,color)),(path,x,actual,color);sample.append(actual);widths.append(sum(all(abs(a-b)<=1 for a,b in zip(image[lineY][px+delta],color)) for delta in range(-6,7)))
   assert 0<widths[0]<widths[1]<widths[2]
   mismatches=sum(any(abs(a-b)>1 for a,b in zip(image[y][x],baseline[2][y][x])) for y in range(h) for x in range(w));assert mismatches/(w*h)<=.005
   records.append({'platform':platform,'file':str(path),'SHA256':hashlib.sha256(path.read_bytes()).hexdigest(),'backend':json.loads((directory/(browser+'-fixture-'+force+'.json')).read_text(encoding='utf8'))['diagnostics']['backend'],'RGB':sample,'widths':widths,'baselineMismatchFraction':mismatches/(w*h),'independentOracle':'Python PNG CRC/filter decode; document12x6 x2/4/6 source boundary+definedRGB/style.fit920. Baseline is additional check only.'})
for platform,base,browsers in [('Windows',OUT/'m1/dpr',['chrome','edge','chromium','firefox','webkit']),('Linux',OUT/'independent-ci/wp05-protocol-browser-evidence/target/wp08/dpr',['chromium','firefox','webkit'])]:
 for browser in browsers:
  for force in ['false','true']:
   info=json.loads((base/(browser+'-'+force+'.json')).read_text(encoding='utf8'))
   for i,step in enumerate(info['steps']):
    dpr=step['dpr'];path=base/(browser+'-'+force+'-dpr'+str(dpr)+'-'+str(i)+'.png');w,h,image=decode(path);b=step['box'];c=step['camera'];x=(1-c['x'])/c['width']*b['width']+b['width']/2;y=(1-c['y'])/c['height']*b['height']+b['height']/2
    if step['image']['capture'].startswith('native full'):px=math.floor((b['x']+x)*w/1280);py=math.floor((b['y']+y)*h/720)
    else:px=math.floor(x*w/b['width']);py=math.floor(y*h/b['height'])
    actual=image[py][px];assert all(abs(a-e)<=1 for a,e in zip(actual,[77,111,155]));assert step['buffer']['width']==step['buffer']['expectedWidth'] and step['buffer']['height']==step['buffer']['expectedHeight'];assert info['errors']==[]
    records.append({'platform':platform,'file':str(path),'actual':actual,'expected':[77,111,155],'buffer':step['buffer'],'dpr':dpr,'scope':info['scope']})
(OUT/'independent-pixel-audit.json').write_text(json.dumps(records,indent=2),encoding='utf8');print('PASS',len(records),'independently decoded PNG checks, borders+full baselines+DPR')
