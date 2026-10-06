# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: malformed-world.spec.ts >> M1 malformed decoded ledger preserves last valid panel and closes connection
- Location: e2e-m1/malformed-world.spec.ts:5:1

# Error details

```
Error: expect(locator).toHaveText(expected) failed

Locator: getByRole('alert')
Expected: "Invalid server message; connection closed"
Error: strict mode violation: getByRole('alert') resolved to 2 elements:
    1) <p role="alert">This browser could not initialize WebGPU or WebGL…</p> aka getByText('This browser could not')
    2) <p role="alert">Invalid server message; connection closed</p> aka getByText('Invalid server message;')

Call log:
  - Expect "toHaveText" getByRole('alert') with timeout 5000ms
  - waiting for getByRole('alert')

```

# Page snapshot

```yaml
- main [ref=e3]:
  - generic [ref=e4]:
    - generic [ref=e5]:
      - heading "OpenHOI4" [level=1] [ref=e6]
      - paragraph [ref=e7]: Local scenario
    - generic [ref=e8]:
      - text: Language
      - combobox "Language" [ref=e9]:
        - option "English" [selected]
        - option "한국어"
  - status [ref=e10]: Disconnected
  - region "Time controls" [ref=e11]:
    - generic [ref=e12]:
      - generic [ref=e13]:
        - term [ref=e14]: Date
        - definition [ref=e15]: 2000-01-01
      - generic [ref=e16]:
        - term [ref=e17]: Hour
        - definition [ref=e18]: "2"
      - generic [ref=e19]:
        - term [ref=e20]: Tick
        - definition [ref=e21]: "2"
      - generic [ref=e22]:
        - term [ref=e23]: Speed
        - definition [ref=e24]: "1"
    - generic [ref=e25]:
      - button "Pause" [disabled] [ref=e26]
      - button "Speed 1" [disabled] [pressed] [ref=e27]: "1"
      - button "Speed 2" [disabled] [ref=e28]: "2"
      - button "Speed 3" [disabled] [ref=e29]: "3"
      - button "Speed 4" [disabled] [ref=e30]: "4"
      - button "Speed 5" [disabled] [ref=e31]: "5"
  - generic [ref=e32]:
    - region "Province map" [ref=e33]:
      - navigation "Map modes" [ref=e34]:
        - button "Ownership" [pressed] [ref=e35] [cursor=pointer]
        - button "Control" [ref=e36] [cursor=pointer]
        - button "Terrain" [ref=e37] [cursor=pointer]
        - button "States" [ref=e38] [cursor=pointer]
        - button "Reset view" [ref=e39] [cursor=pointer]
      - alert [ref=e41]: This browser could not initialize WebGPU or WebGL2. Try a browser that supports WebGL2.
      - generic [ref=e42]:
        - generic [ref=e43]:
          - text: "Province:"
          - strong [ref=e44]: —
        - generic [ref=e45]: Click to select · Drag to pan · Scroll to zoom
        - button "Clear selection" [disabled] [ref=e46]
    - complementary [ref=e47]:
      - generic [ref=e48]:
        - region "Country" [ref=e49]:
          - heading "Country" [level=2] [ref=e50]
          - navigation [ref=e51]:
            - button "Northern Test Nation" [pressed] [ref=e52] [cursor=pointer]
            - button "Southern Test Nation" [ref=e53] [cursor=pointer]
          - heading "Northern Test Nation" [level=3] [ref=e54]
          - generic [ref=e55]:
            - generic [ref=e56]:
              - term [ref=e57]: Tag
              - definition [ref=e58]: NTH
            - generic [ref=e59]:
              - term [ref=e60]: Capital province
              - definition [ref=e61]: "10"
            - generic [ref=e62]:
              - term [ref=e63]: Government
              - definition [ref=e64]: Test Republic
          - heading "Ideology support (ratio)" [level=4] [ref=e65]
          - generic [ref=e66]:
            - generic [ref=e67]:
              - term [ref=e68]: Civic
              - definition [ref=e69]: "0.75"
            - generic [ref=e70]:
              - term [ref=e71]: Local
              - definition [ref=e72]: "0.25"
        - region "State" [ref=e73]:
          - heading "State" [level=2] [ref=e74]
          - navigation [ref=e75]:
            - button "Northern Test State" [pressed] [ref=e76] [cursor=pointer]
            - button "Southern Test State" [ref=e77] [cursor=pointer]
          - heading "Northern Test State" [level=3] [ref=e78]
          - generic [ref=e79]:
            - generic [ref=e80]:
              - term [ref=e81]: Owner
              - definition [ref=e82]: Northern Test Nation
            - generic [ref=e83]:
              - term [ref=e84]: Population
              - definition [ref=e85]: "1000"
            - generic [ref=e86]:
              - term [ref=e87]: Steel
              - definition [ref=e88]: "2"
            - generic [ref=e89]:
              - term [ref=e90]: Industry
              - definition [ref=e91]: "1"
          - group [ref=e92]:
            - generic "Show ledger for Infrastructure" [ref=e93] [cursor=pointer]:
              - text: Infrastructure
              - strong [ref=e94]: "1"
          - table [ref=e95]:
            - rowgroup [ref=e96]:
              - row [ref=e97]:
                - columnheader "Province" [ref=e98]
                - columnheader "Owner" [ref=e99]
                - columnheader "Controller" [ref=e100]
            - rowgroup [ref=e101]:
              - row [ref=e102]:
                - cell "10" [ref=e103]
                - cell "Northern Test Nation" [ref=e104]
                - cell "Northern Test Nation" [ref=e105]
              - row [ref=e106]:
                - cell "20" [ref=e107]
                - cell "Northern Test Nation" [ref=e108]
                - cell "Southern Test Nation" [ref=e109]
  - alert [ref=e110]: Invalid server message; connection closed
  - link "Noto Sans KR · SIL Open Font License 1.1" [ref=e112] [cursor=pointer]:
    - /url: /fonts/OFL.txt
```

# Test source

```ts
  1  | import { expect,test } from '@playwright/test';
  2  | import { encode,decode } from '@msgpack/msgpack';
  3  | import { mkdirSync,writeFileSync } from 'node:fs';
  4  | const evidence=process.env.OH_MALFORMED_EVIDENCE??'../target/wp09/malformed';mkdirSync(evidence,{recursive:true});
  5  | test('M1 malformed decoded ledger preserves last valid panel and closes connection',async({page},info)=>{
  6  |  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));let worlds=0;
  7  |  await page.routeWebSocket('**/ws',route=>{const server=route.connectToServer();server.onMessage(data=>{
  8  |   const message=decode(data as Buffer) as {type:string;world:{states:{infrastructure:{final_value:unknown}}[]} };
  9  |   if(message.type==='WorldResult'&&message.world&&++worlds>=3){message.world.states[0].infrastructure.final_value={invalid:true};route.send(Buffer.from(encode(message)));}else route.send(data);
  10 |  });});
  11 |  await page.goto('/');await expect(page.getByTestId('country-panel')).toContainText('Northern Test Nation');
  12 |  await expect(page.getByTestId('connection')).toHaveText('Disconnected');await expect(page.getByTestId('pause')).toBeDisabled();
> 13 |  await expect(page.getByRole('alert')).toHaveText('Invalid server message; connection closed');expect(errors).toEqual([]);
     |                                        ^ Error: expect(locator).toHaveText(expected) failed
  14 |  await expect(page.getByTestId('state-panel').locator('.ledger-value strong')).toHaveText('1');
  15 |  await page.screenshot({path:`${evidence}/${info.project.name}-world-malformed.png`,fullPage:true});
  16 |  await page.getByRole('combobox').selectOption('ko');await expect(page.getByRole('alert')).toHaveText('잘못된 서버 메시지로 연결을 종료했습니다');
  17 |  writeFileSync(`${evidence}/${info.project.name}-world-malformed.json`,JSON.stringify({errors,worlds,closed:true},null,2));
  18 | });
  19 | 
```