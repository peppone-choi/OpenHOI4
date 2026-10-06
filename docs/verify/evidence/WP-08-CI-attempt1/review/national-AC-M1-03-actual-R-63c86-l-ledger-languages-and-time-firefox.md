# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: national.spec.ts >> AC-M1-03 actual Rust M1 panels, separate owner/control, ledger, languages and time
- Location: e2e-m1/national.spec.ts:6:1

# Error details

```
Error: expect(received).toEqual(expected) // deep equality

- Expected  - 1
+ Received  + 3

- Array []
+ Array [
+   "can't access property \"getSupportedExtensions\", this.gl is null",
+ ]
```

# Page snapshot

```yaml
- main [ref=e3]:
  - generic [ref=e4]:
    - generic [ref=e5]:
      - heading "OpenHOI4" [level=1] [ref=e6]
      - paragraph [ref=e7]: 로컬 시나리오
    - generic [ref=e8]:
      - text: 언어
      - combobox "언어" [ref=e9]:
        - option "English"
        - option "한국어" [selected]
  - status [ref=e10]: 연결됨
  - region "시간 제어" [ref=e11]:
    - generic [ref=e12]:
      - generic [ref=e13]:
        - term [ref=e14]: 날짜
        - definition [ref=e15]: 2001-10-11
      - generic [ref=e16]:
        - term [ref=e17]: 시각
        - definition [ref=e18]: "14"
      - generic [ref=e19]:
        - term [ref=e20]: 틱
        - definition [ref=e21]: "15590"
      - generic [ref=e22]:
        - term [ref=e23]: 속도
        - definition [ref=e24]: "5"
    - generic [ref=e25]:
      - button "재개" [active] [ref=e26] [cursor=pointer]
      - button "속도 1" [ref=e27] [cursor=pointer]: "1"
      - button "속도 2" [ref=e28] [cursor=pointer]: "2"
      - button "속도 3" [ref=e29] [cursor=pointer]: "3"
      - button "속도 4" [ref=e30] [cursor=pointer]: "4"
      - button "속도 5" [pressed] [ref=e31] [cursor=pointer]: "5"
  - generic [ref=e32]:
    - region "프로빈스 지도" [ref=e33]:
      - navigation "지도 모드" [ref=e34]:
        - button "소유" [pressed] [ref=e35] [cursor=pointer]
        - button "통제" [ref=e36] [cursor=pointer]
        - button "지형" [ref=e37] [cursor=pointer]
        - button "주" [ref=e38] [cursor=pointer]
        - button "시점 초기화" [ref=e39] [cursor=pointer]
      - alert [ref=e41]: 이 브라우저에서 WebGPU와 WebGL2를 초기화할 수 없습니다. WebGL2를 지원하는 브라우저를 사용하세요.
      - generic [ref=e42]:
        - generic [ref=e43]:
          - text: "프로빈스:"
          - strong [ref=e44]: —
        - generic [ref=e45]: 클릭으로 선택 · 드래그로 이동 · 스크롤로 확대
        - button "선택 해제" [disabled] [ref=e46]
    - complementary [ref=e47]:
      - generic [ref=e48]:
        - region "국가" [ref=e49]:
          - heading "국가" [level=2] [ref=e50]
          - navigation [ref=e51]:
            - button "북부 시험국" [pressed] [ref=e52] [cursor=pointer]
            - button "남부 시험국" [ref=e53] [cursor=pointer]
          - heading "북부 시험국" [level=3] [ref=e54]
          - generic [ref=e55]:
            - generic [ref=e56]:
              - term [ref=e57]: 태그
              - definition [ref=e58]: NTH
            - generic [ref=e59]:
              - term [ref=e60]: 수도 프로빈스
              - definition [ref=e61]: "10"
            - generic [ref=e62]:
              - term [ref=e63]: 정부
              - definition [ref=e64]: 시험 공화정
          - heading "이념 지지율 (비율)" [level=4] [ref=e65]
          - generic [ref=e66]:
            - generic [ref=e67]:
              - term [ref=e68]: 시민
              - definition [ref=e69]: "0.75"
            - generic [ref=e70]:
              - term [ref=e71]: 지역
              - definition [ref=e72]: "0.25"
        - region "주" [ref=e73]:
          - heading "주" [level=2] [ref=e74]
          - navigation [ref=e75]:
            - button "북부 시험주" [pressed] [ref=e76] [cursor=pointer]
            - button "남부 시험주" [ref=e77] [cursor=pointer]
          - heading "북부 시험주" [level=3] [ref=e78]
          - generic [ref=e79]:
            - generic [ref=e80]:
              - term [ref=e81]: 소유국
              - definition [ref=e82]: 북부 시험국
            - generic [ref=e83]:
              - term [ref=e84]: 인구
              - definition [ref=e85]: "1000"
            - generic [ref=e86]:
              - term [ref=e87]: 철강
              - definition [ref=e88]: "2"
            - generic [ref=e89]:
              - term [ref=e90]: 공업 시설
              - definition [ref=e91]: "1"
          - group [ref=e92]:
            - generic "인프라 수치 원장 보기" [ref=e93] [cursor=pointer]:
              - text: 인프라
              - strong [ref=e94]: "1"
            - generic [ref=e95]:
              - heading "수치 원장" [level=2] [ref=e96]
              - paragraph [ref=e97]: "틱: 15590"
              - generic [ref=e98]:
                - generic [ref=e99]:
                  - term [ref=e100]: 기본값
                  - definition [ref=e101]: "1"
                - generic [ref=e102]:
                  - term [ref=e103]: 최종값
                  - definition [ref=e104]: "1"
              - table [ref=e105]:
                - rowgroup [ref=e106]:
                  - row [ref=e107]:
                    - columnheader "기여 항목" [ref=e108]
                    - columnheader "연산" [ref=e109]
                    - columnheader "값" [ref=e110]
                    - columnheader "누적값" [ref=e111]
                - rowgroup [ref=e112]:
                  - row [ref=e113]:
                    - 'cell "인프라 출처: 기본값" [ref=e114]':
                      - text: 인프라
                      - generic [ref=e115]: "출처: 기본값"
                    - cell "기본값" [ref=e116]
                    - cell "1" [ref=e117]
                    - cell "1" [ref=e118]
          - table [ref=e119]:
            - rowgroup [ref=e120]:
              - row [ref=e121]:
                - columnheader "프로빈스" [ref=e122]
                - columnheader "소유국" [ref=e123]
                - columnheader "통제국" [ref=e124]
            - rowgroup [ref=e125]:
              - row [ref=e126]:
                - cell "10" [ref=e127]
                - cell "북부 시험국" [ref=e128]
                - cell "북부 시험국" [ref=e129]
              - row [ref=e130]:
                - cell "20" [ref=e131]
                - cell "북부 시험국" [ref=e132]
                - cell "남부 시험국" [ref=e133]
  - link "Noto Sans KR · SIL Open Font License 1.1" [ref=e135] [cursor=pointer]:
    - /url: /fonts/OFL.txt
```

# Test source

```ts
  1  | import { expect,test } from '@playwright/test';
  2  | import { encode,decode } from '@msgpack/msgpack';
  3  | import { mkdirSync,writeFileSync } from 'node:fs';
  4  | import type { ServerMessage,ClientMessage } from '../src/proto/protocol';
  5  | const evidence=process.env.OH_E2E_EVIDENCE??'../target/wp09/e2e';mkdirSync(evidence,{recursive:true});
  6  | test('AC-M1-03 actual Rust M1 panels, separate owner/control, ledger, languages and time',async({page,request,browser},info)=>{
  7  |  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('/');
  8  |  await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  9  |  await expect(page.getByTestId('country-panel')).toContainText('Test Republic');
  10 |  await expect(page.getByTestId('state-panel')).toContainText('Northern Test State');
  11 |  await expect(page.getByTestId('province-20')).toHaveText('20Northern Test NationSouthern Test Nation');
  12 |  await page.getByTestId('pause').click();await expect(page.getByTestId('pause')).toHaveText('Resume');
  13 |  const tick=await page.getByTestId('tick').textContent();await page.waitForTimeout(400);await expect(page.getByTestId('tick')).toHaveText(tick!);
  14 |  const panel=page.getByTestId('state-panel');await panel.getByRole('button',{name:'Northern Test State'}).click();
  15 |  await panel.getByText('Infrastructure',{exact:true}).click();
  16 |  await expect(panel.locator('.ledger-value strong')).toHaveText('1');await expect(panel.locator('.ledger-tooltip tbody tr')).toHaveCount(1);
  17 |  await expect(panel.locator('.ledger-tooltip tbody tr td').nth(2)).toHaveText('1');await expect(panel.locator('.ledger-tooltip tbody tr td').nth(3)).toHaveText('1');
  18 |  const worldTick=await page.getByTestId('national-panels').getAttribute('data-world-tick');
  19 |  await expect(panel.locator('.ledger-tooltip')).toContainText(`Tick: ${worldTick}`);
  20 |  for(const speed of [1,2,3,4,5]){await page.getByTestId(`speed-${speed}`).click();await expect(page.getByTestId('speed')).toHaveText(String(speed));}
  21 |  await page.screenshot({path:`${evidence}/${info.project.name}-m1-en.png`,fullPage:true});
  22 |  await page.getByTestId('country-panel').getByRole('button',{name:'Southern Test Nation'}).click();await expect(page.getByTestId('country-panel')).toContainText('Test Council');
  23 |  await panel.getByRole('button',{name:'Southern Test State'}).click();await expect(panel.locator('.ledger-value strong')).toHaveText('0');
  24 |  await page.getByRole('combobox').selectOption('ko');await expect(page.getByTestId('country-panel')).toContainText('남부 시험국');await expect(panel).toContainText('남부 시험주');
  25 |  await panel.getByRole('button',{name:'북부 시험주'}).click();await expect(page.getByTestId('province-20')).toHaveText('20북부 시험국남부 시험국');
  26 |  await page.screenshot({path:`${evidence}/${info.project.name}-m1-ko.png`,fullPage:true});
  27 |  await page.getByTestId('pause').click();await expect(page.getByTestId('tick')).not.toHaveText(tick!);await page.getByTestId('pause').click();
  28 |  for(const lang of ['en','ko'])for(const file of ['map.ftl','national.ftl']){const f=await request.get(`/pack/localisation/${lang}/${file}`);expect(f.status()).toBe(200);expect(f.headers()['content-type']).toContain('text/plain');}
> 29 |  expect(errors).toEqual([]);writeFileSync(`${evidence}/${info.project.name}-m1.json`,JSON.stringify({browser:browser.version(),project:info.project.name,scenario:'m1',seed:'1',errors,worldTick,viewport:{width:1280,height:720},render:'actual Rust M1 DOM panels; WP08 GPU map pending'},null,2));
     |                 ^ Error: expect(received).toEqual(expected) // deep equality
  30 | });
  31 | test('REQ-NAT-01 MAP-08 actual query metadata, dense mapping, colors and refusal',async({page})=>{
  32 |  await page.goto('/');
  33 |  const messages:ClientMessage[]=[{type:'Hello',protocol_version:'m0-v1'},{type:'Query',request:'before',kind:'world'},{type:'Join',session:'local',nation:null},{type:'Command',sequence:'1',command:{type:'Pause',paused:true}},{type:'Query',request:'world',kind:'world'},{type:'Query',request:'north',kind:'nation:1'},{type:'Query',request:'state',kind:'state:1'},{type:'Query',request:'missing',kind:'state:65535'},{type:'Query',request:'bad',kind:'nation:abc'},{type:'Query',request:'scope',kind:'economy'}];
  34 |  const frames=await page.evaluate(bytes=>new Promise<number[][]>(resolve=>{const frames:number[][]=[];const ws=new WebSocket(`ws://${location.host}/ws`);ws.binaryType='arraybuffer';ws.onopen=()=>bytes.forEach(b=>ws.send(Uint8Array.from(b)));ws.onmessage=e=>frames.push(Array.from(new Uint8Array(e.data)));setTimeout(()=>{ws.close();resolve(frames);},500);}),messages.map(m=>Array.from(encode(m))));
  35 |  const values=frames.map(b=>decode(Uint8Array.from(b))) as ServerMessage[];
  36 |  const result=values.find(m=>m.type==='WorldResult'&&m.request==='world');expect(result?.type).toBe('WorldResult');if(result?.type!=='WorldResult'||!result.world)throw new Error('missing actual world');
  37 |  const world=result.world;expect(world.map_id).toBe('testland');expect(world.province_ids).toEqual([10,20,30,40,50,60]);expect(world.width).toBe(8);expect(world.height).toBe(6);
  38 |  expect(world.provinces.find(p=>p.id===20)).toMatchObject({owner:1,controller:2,owner_color:[40,100,180],controller_color:[180,70,40],state_color:[140,95,170],terrain_color:[155,135,95]});
  39 |  expect(world.provinces.find(p=>p.id===50)).toMatchObject({owner:null,controller:null});
  40 |  expect(world.states[0].infrastructure).toMatchObject({base:'1',final_value:'1',tick:world.tick,entries:[{value:'1',accumulated:'1'}]});
  41 |  for(const request of ['missing','bad'])expect(values).toContainEqual({type:'WorldResult',request,supported:false,reason_key:'unknown-entity',world:null});
  42 |  expect(values).toContainEqual({type:'WorldResult',request:'before',supported:false,reason_key:'not-joined',world:null});
  43 |  expect(values).toContainEqual({type:'QueryResult',request:'scope',supported:false,reason_key:'unsupported-query',state:null});
  44 | });
  45 | 
```