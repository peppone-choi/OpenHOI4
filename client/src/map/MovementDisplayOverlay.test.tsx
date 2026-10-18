import {expect,test} from 'vitest';
import {readdirSync} from 'node:fs';
import {renderToStaticMarkup} from 'react-dom/server';
import {MovementOverlay} from './MovementDisplayOverlay';
import type {MovementGeometry} from './movementOverlay';
import {Localization,translatorFor} from '../i18n';

test('map module stems are unambiguous on case-insensitive filesystems',()=>{
 const stems=readdirSync(new URL('.',import.meta.url)).filter(name=>/\.tsx?$/.test(name)).map(name=>name.replace(/\.tsx?$/,'').toLowerCase());
 expect(new Set(stems).size).toBe(stems.length);
});

test('moving markers keep compact labels and localized server remaining time accessible',()=>{
 const geometry:MovementGeometry={arrows:[],excluded:new Set(['0']),moving:[{division:'0',nation:1,province:1,color:[1,2,3],x:1,y:1,left:25,top:25,ratio:.5,remaining:'100'}]};
 for(const language of ['en','ko'] as const){
  const t=translatorFor(language);
  const html=renderToStaticMarkup(<Localization.Provider value={{language,t}}><MovementOverlay geometry={geometry} selected={['0']} onActivate={()=>{}} onContextPick={()=>false} host={{current:null}}/></Localization.Provider>);
  expect(html).toContain('>0</button>');
  expect(html).toContain(`title="${t('movement-remaining',{remaining:'100'})}"`);
  expect(html).toContain(`aria-label="${t('map-moving-division',{division:'0',nation:1,remaining:'100'})}"`);
  expect(html).toContain('aria-pressed="true"');
 }
});
