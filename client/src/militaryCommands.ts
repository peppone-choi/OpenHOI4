import type {MilitaryCommand,MilitaryView,ServerMessage} from './proto/protocol';
export type TrainingCommand=Extract<MilitaryCommand,{type:'Train'|'Cancel'}>;
export type TrainingFeedback={status:'idle'|'pending'|'refreshing'|'success'|'rejected';reasonKey:string|null};
export const emptyTrainingFeedback=():TrainingFeedback=>({status:'idle',reasonKey:null});
/** Command authority and one pending action; all gameplay stays on the server. */
export class MilitaryCommands {
  private socket:object|null=null;
  private nation:number|null=null;
  private base:MilitaryView|null=null;
  private authorityAfter=0n;
  private pending:{sequence:string;barrier:bigint|null;outcome:{accepted:boolean;reason_key:string|null}|null}|null=null;
  feedback=emptyTrainingFeedback();
  begin(socket:object,nation:number|null,querySerial=0n){this.close();this.socket=socket;this.nation=nation;this.authorityAfter=querySerial;}
  close(){this.socket=null;this.nation=null;this.base=null;this.pending=null;this.feedback=emptyTrainingFeedback();}
  end(socket:object){if(socket===this.socket)this.close();}
  setAuthority(socket:object,base:MilitaryView|null,querySerial:bigint){
    if(socket!==this.socket||querySerial<=this.authorityAfter)return;
    this.base=base;
    const pending=this.pending;
    if(base&&pending?.outcome&&pending.barrier!==null&&querySerial>pending.barrier){
      this.feedback={status:pending.outcome.accepted?'success':'rejected',reasonKey:pending.outcome.reason_key};this.pending=null;
    }
  }
  can(socket:object,nationFilter:number|null,command:TrainingCommand):boolean{
    if(socket!==this.socket||this.nation===null||!this.base||this.pending||(nationFilter!==null&&nationFilter!==this.nation))return false;
    if(command.type==='Train')return this.base.templates.some(t=>t.template===command.template);
    return this.base.jobs.some(j=>j.id===command.job&&j.nation===this.nation&&['Pending','Training','Ready'].includes(j.status));
  }
  issue(socket:object,nationFilter:number|null,command:TrainingCommand,transmit:()=>string|null):string|null{
    if(!this.can(socket,nationFilter,command))return null;
    const sequence=transmit();if(sequence===null)return null;
    this.pending={sequence,barrier:null,outcome:null};this.feedback={status:'pending',reasonKey:null};return sequence;
  }
  accept(socket:object,message:Extract<ServerMessage,{type:'CommandResult'}>,querySerial:bigint):boolean{
    if(socket!==this.socket||!this.pending||message.sequence!==this.pending.sequence||this.pending.outcome)return false;
    this.pending.barrier=querySerial;this.pending.outcome={accepted:message.accepted,reason_key:message.reason_key};
    this.feedback={status:'refreshing',reasonKey:message.reason_key};return true;
  }
}
