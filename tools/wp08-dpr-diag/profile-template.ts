import {test as base} from '../../client/node_modules/@playwright/test';
import {mkdirSync,writeFileSync} from 'node:fs';
import {performance} from 'node:perf_hooks';
import {resolve} from 'node:path';
let active:any=null;let restore=()=>{};
const output=resolve(process.env.OH_DPR_PROFILE!);mkdirSync(output,{recursive:true});
const flush=()=>{if(active)writeFileSync(active.path,JSON.stringify(active,null,2)+'\n');};
export function begin(info:any,force:boolean,browser:any){const connection=(browser as any)._connection,original=connection.sendMessageToServer;
 if(typeof original!=='function')throw new Error('Pinned SDK Connection.sendMessageToServer not available');
 connection.sendMessageToServer=async function(object:any,method:string,params:any,options:any){
  const profile=active,start=performance.now(),row=profile?{stage:profile.stage,startMs:start-profile.started,type:object._type,method,cdpMethod:params?.method,apiName:options?.apiName,title:options?.title,selector:params?.selector,expressionKind:params?.expression?.includes('__dprLife')?'lifetime':params?.expression?.includes('getBoundingClientRect')?'geometry':params?.expression?.includes('readPixels')?'GL readPixels':params?.expression?.includes('devicePixelRatio')?'DPR/buffer':params?.expression?.includes('data-camera')?'camera':undefined,ms:0,error:null as string|null}:null;
  if(row)profile.rpc.push(row);
  try{return await Reflect.apply(original,this,[object,method,params,options]);}catch(e){if(row)row.error=String(e);throw e;}finally{if(row)row.ms=performance.now()-start;}
 };
 restore=()=>{connection.sendMessageToServer=original;};active={path:resolve(output,`${info.project.name}-${force}-profile.json`),project:info.project.name,force,timeout:info.timeout,started:performance.now(),stage:'setup',phases:[],rpc:[],cpu:[],scope:'Pinned Playwright1.63 Connection client-driver request timing only. Original results/options/errors forwarded unchanged; no browser GPU API wrapper or additional browser reads.'};phase('setup');}
export function phase(stage:string){if(!active)return;active.phases.push({stage,ms:performance.now()-active.started,rpcCount:active.rpc.length});active.stage=stage;flush();}
export function profilePng<T>(fn:()=>T):T{const start=performance.now();try{return fn();}finally{active?.cpu.push({kind:'full PNG decode',stage:active.stage,ms:performance.now()-start});}}
export function finish(info:any){phase('report-after-body');if(active){active.reportedStatusAfterBody=info.status;flush();}restore();active=null;}
export const test=base;test.afterEach(async({},info)=>{finish(info);});
