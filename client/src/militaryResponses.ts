import type {MilitaryView,ServerMessage} from './proto/protocol';

export type MilitaryDisplay={view:MilitaryView|null;status:'loading'|'ready'|'stale'|'unsupported'|'disconnected';reasonKey:string|null};

/** Correlation for an open panel, without transport validation or game rules. */
export class MilitaryResponses {
  private socket:object|null=null;
  private epoch=0;
  private serial=0n;
  private pending:{request:string;epoch:number}|null=null;
  private dirty=false;
  begin(socket:object){this.close();this.socket=socket;}
  close(){this.epoch++;this.socket=null;this.pending=null;this.dirty=false;}
  end(socket:object):boolean{if(socket!==this.socket)return false;this.close();return true;}
  issue(socket:object,transmit:(request:string)=>boolean):string|null{
    if(socket!==this.socket)return null;
    if(this.pending){this.dirty=true;return null;}
    const next=this.serial+1n,request=`military:${next}`;
    if(!transmit(request))return null;
    this.serial=next;this.pending={request,epoch:this.epoch};this.dirty=false;
    return request;
  }
  followUp(socket:object,transmit:(request:string)=>boolean):string|null{
    if(socket!==this.socket||this.pending||!this.dirty)return null;
    return this.issue(socket,transmit);
  }
  accept(socket:object,message:Extract<ServerMessage,{type:'MilitaryResult'|'QueryResult'}>):MilitaryDisplay|null{
    if(message.type==='QueryResult'&&(message.supported||message.state!==null))return null;
    if(socket!==this.socket||!this.pending||this.pending.epoch!==this.epoch||message.request!==this.pending.request)return null;
    this.pending=null;
    return {view:message.type==='MilitaryResult'?message.military:null,status:message.supported?'ready':'unsupported',reasonKey:message.reason_key};
  }
}
