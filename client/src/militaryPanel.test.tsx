import {readFileSync} from 'node:fs';
import {renderToStaticMarkup} from 'react-dom/server';
import {expect,test} from 'vitest';
import {MilitaryPanel} from './components/MilitaryPanel';
import {Localization,translatorFor} from './i18n';
import type {MilitaryView} from './proto/protocol';
import type {MilitaryDisplay} from './militaryResponses';
const view=JSON.parse(readFileSync(new URL('../../target/wp16/military-wire-fixture.json',import.meta.url),'utf8')).military as MilitaryView;
function render(display:MilitaryDisplay,language:'ko'|'en'='en',nationId:number|null=null){return renderToStaticMarkup(<Localization.Provider value={{language,t:translatorFor(language)}}><MilitaryPanel display={display} world={null} nationId={nationId} onNationSelect={()=>{}} onClose={()=>{}}/></Localization.Provider>);}
test('actual Rust military projection displays current authority fields in both languages',()=>{
  for(const language of ['ko','en'] as const){
    const html=render({view,status:'ready',reasonKey:null},language);
    for(const section of ['templates','armies','divisions','jobs','background','pending'])expect(html).toContain(`data-testid="military-${section}"`);
    expect(html).toContain(view.templates[0].template);
    expect(html).toContain(view.jobs[0].reserved_manpower);
    expect(html).toContain(`data-bits="${view.templates[0].normal.speed_kmh.bits}"`);
    expect(html).not.toMatch(/Unknown message|알 수 없는 메시지/);
    expect(html).not.toContain('MilitaryCommand');
  }
});
test('canonical large strings, nullable ownership and server order survive without gameplay computation',()=>{
  const huge=structuredClone(view);huge.jobs[0].id='18446744073709551615';huge.jobs[0].reserved_manpower='9007199254740993';huge.jobs[0].start_tick=null;huge.jobs[0].division=null;
  const html=render({view:huge,status:'ready',reasonKey:null});
  expect(html).toContain('18446744073709551615');expect(html).toContain('9007199254740993');expect(html).toContain('None');
  const filtered=render({view,status:'ready',reasonKey:null},'en',65535);
  expect(filtered).not.toContain('data-military-job=');expect(filtered).toContain(view.templates[0].template);
});
test('loading, stale, disconnected, unsupported and empty states have distinct localized labels',()=>{
  for(const status of ['loading','stale','disconnected','unsupported'] as const){
    expect(render({view:status==='stale'?view:null,status,reasonKey:null})).toContain(`data-status="${status}"`);
  }
  const empty={...view,templates:[],armies:[],divisions:[],jobs:[],background:[],pending:[]};
  expect(render({view:empty,status:'ready',reasonKey:null})).toContain('No military records');
  expect(render({view:null,status:'unsupported',reasonKey:'unsupported-query'})).toContain('data-status="unsupported"');
});
