import type {MilitaryCommand,MilitaryView,ServerMessage,WorldView} from './proto/protocol';
import {landSet} from './worldAuthority';
export type TrainingCommand=Extract<MilitaryCommand,{type:'Train'|'Cancel'|'Deploy'}>;
const u64=(value:unknown):value is string=>typeof value==='string'&&/^(0|[1-9][0-9]*)$/.test(value)&&value.length<=20&&BigInt(value)<=18446744073709551615n;
export type TrainingFeedback={status:'idle'|'pending'|'refreshing'|'success'|'rejected';reasonKey:string|null};
export const emptyTrainingFeedback=():TrainingFeedback=>({status:'idle',reasonKey:null});
/** Command authority and one pending action; all gameplay stays on the server. */
export class MilitaryCommands {
  private socket:object|null=null;
  private nation:number|null=null;
  private base:MilitaryView|null=null;
  private authorityAfter=0n;
  private baseSerial=0n;
  private world:WorldView|null=null;
  private land:Set<number>|null=null;
  private worldSerial=0n;
  private worldAfter=0n;
  private pending:{sequence:string;deploy:boolean;barrier:bigint|null;worldBarrier:bigint|null;outcome:{accepted:boolean;reason_key:string|null}|null}|null=null;
  feedback=emptyTrainingFeedback();
  begin(socket:object,nation:number|null,querySerial=0n,worldSerial=0n){this.close();this.socket=socket;this.nation=nation;this.authorityAfter=querySerial;this.worldAfter=worldSerial;}
  close(){this.socket=null;this.nation=null;this.base=null;this.world=null;this.land=null;this.pending=null;this.baseSerial=0n;this.worldSerial=0n;this.feedback=emptyTrainingFeedback();}
  end(socket:object){if(socket===this.socket)this.close();}
  setAuthority(socket:object,base:MilitaryView|null,querySerial:bigint){
    if(socket!==this.socket||querySerial<=this.authorityAfter)return;
    this.base=base;
    this.baseSerial=querySerial;this.finish();
  }
  setWorldAuthority(socket:object,world:WorldView|null,querySerial:bigint){
    if(socket!==this.socket)return;
    if(world&&querySerial<=this.worldAfter)return;
    this.land=world?landSet(world):null;this.world=this.land?world:null;this.worldSerial=querySerial;this.finish();
  }
  private finish(){
    const pending=this.pending;
    if(this.base&&pending?.outcome&&pending.barrier!==null&&this.baseSerial>pending.barrier&&(!pending.deploy||(this.world&&pending.worldBarrier!==null&&this.worldSerial>pending.worldBarrier))){
      this.feedback={status:pending.outcome.accepted?'success':'rejected',reasonKey:pending.outcome.reason_key};this.pending=null;
    }
  }
  can(socket:object,nationFilter:number|null,command:TrainingCommand):boolean{
    if(socket!==this.socket||this.nation===null||!this.base||this.pending||(nationFilter!==null&&nationFilter!==this.nation))return false;
    if(command.type==='Train')return typeof command.template==='string'&&this.base.templates.some(t=>t.template===command.template);
    if(!u64(command.job))return false;
    const job=this.base.jobs.find(j=>j.id===command.job&&j.nation===this.nation);
    if(command.type==='Cancel')return !!job&&['Pending','Training','Ready'].includes(job.status);
    if(command.type!=='Deploy')return false;
    return !!job&&job.status==='Ready'&&u64(command.army)&&this.base.armies.some(a=>a.id===command.army&&a.nation===this.nation)&&Number.isInteger(command.province)&&command.province>=0&&command.province<=65535&&typeof command.allow_understrength==='boolean'&&!!this.land?.has(command.province)&&this.world?.provinces.find(p=>p.id===command.province)?.controller===this.nation;
  }
  issue(socket:object,nationFilter:number|null,command:TrainingCommand,transmit:()=>string|null):string|null{
    if(!this.can(socket,nationFilter,command))return null;
    const sequence=transmit();if(sequence===null)return null;
    this.pending={sequence,deploy:command.type==='Deploy',barrier:null,worldBarrier:null,outcome:null};this.feedback={status:'pending',reasonKey:null};return sequence;
  }
  accept(socket:object,message:Extract<ServerMessage,{type:'CommandResult'}>,querySerial:bigint,worldSerial=0n):boolean{
    if(socket!==this.socket||!this.pending||message.sequence!==this.pending.sequence||this.pending.outcome)return false;
    this.pending.barrier=querySerial;this.pending.worldBarrier=worldSerial;this.pending.outcome={accepted:message.accepted,reason_key:message.reason_key};
    this.world=null;this.land=null;
    this.feedback={status:'refreshing',reasonKey:message.reason_key};return true;
  }
}
