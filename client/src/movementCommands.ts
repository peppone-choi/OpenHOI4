import type {MilitaryView,MovementCommand,MovementView,ServerMessage} from './proto/protocol';
export type MovementDisplay={view:MovementView|null;status:'loading'|'ready'|'stale'|'unsupported'|'disconnected';reasonKey:string|null};
export type MovementFeedback={division:string;status:'pending'|'accepted'|'rejected'|'not-sent';reasonKey:string|null};
/** Correlates a single open-panel query with coalesced refresh, not game rules. */
export class MovementResponses {
 private socket:object|null=null;
 private epoch=0;
 private serial=0n;
 private pending:{request:string;epoch:number}|null=null;
 private dirty=false;
 get issuedSerial(){return this.serial;}
 begin(socket:object){this.close();this.socket=socket;}
 close(){this.epoch++;this.socket=null;this.pending=null;this.dirty=false;}
 end(socket:object){if(socket!==this.socket)return false;this.close();return true;}
 issue(socket:object,transmit:(request:string)=>boolean):string|null{
  if(socket!==this.socket)return null;
  if(this.pending){this.dirty=true;return null;}
  const next=this.serial+1n,request=`movement:${next}`;
  if(!transmit(request))return null;
  this.serial=next;this.pending={request,epoch:this.epoch};this.dirty=false;return request;
 }
 followUp(socket:object,transmit:(request:string)=>boolean):string|null{
  return socket===this.socket&&!this.pending&&this.dirty?this.issue(socket,transmit):null;
 }
 accept(socket:object,message:Extract<ServerMessage,{type:'MovementResult'|'QueryResult'}>):MovementDisplay|null{
  if(message.type==='QueryResult'&&(message.supported||message.state!==null))return null;
  if(socket!==this.socket||!this.pending||this.pending.epoch!==this.epoch||message.request!==this.pending.request)return null;
  this.pending=null;
  return {view:message.type==='MovementResult'?message.movement:null,status:message.supported?'ready':'unsupported',reasonKey:message.reason_key};
 }
}
/** Own roster authority, shared-ledger sends and the post-final-ACK query barrier. */
export class MovementCommands {
 private socket:object|null=null;
 private nation:number|null=null;
 private military:MilitaryView|null=null;
 private pending:Map<string,MovementFeedback>|null=null;
 private barrier:bigint|null=null;
 feedback:MovementFeedback[]=[];
 get busy(){return this.pending!==null;}
 begin(socket:object,nation:number|null){this.close();this.socket=socket;this.nation=nation;}
 close(){this.socket=null;this.nation=null;this.military=null;this.pending=null;this.barrier=null;this.feedback=[];}
 end(socket:object){if(socket===this.socket)this.close();}
 setMilitary(socket:object,view:MilitaryView|null){if(socket===this.socket)this.military=view;}
 issue(socket:object,ids:readonly string[],destination:number|null,transmit:(command:MovementCommand)=>string|null):boolean{
  if(socket!==this.socket||this.nation===null||!this.military||this.busy||!ids.length)return false;
  const own=new Set(this.military.divisions.filter(d=>d.nation===this.nation).map(d=>d.id));
  if(ids.some(id=>!own.has(id)))return false;
  const sorted=[...new Set(ids)].sort((a,b)=>BigInt(a)<BigInt(b)?-1:BigInt(a)>BigInt(b)?1:0);
  const pending=new Map<string,MovementFeedback>();let stopped=false;
  this.feedback=sorted.map(division=>{
   const row:MovementFeedback={division,status:'not-sent',reasonKey:null};
   if(!stopped){
    const command:MovementCommand=destination===null?{type:'Stop',division}:{type:'Move',division,destination};
    const sequence=transmit(command);
    if(sequence===null)stopped=true;else {row.status='pending';pending.set(sequence,row);}
   }return row;
  });
  this.pending=pending.size?pending:null;this.barrier=null;return true;
 }
 /** True only on the last ACK: caller schedules a fresh movement query then. */
 accept(socket:object,message:Extract<ServerMessage,{type:'CommandResult'}>,querySerial:bigint):boolean{
  if(socket!==this.socket||!this.pending)return false;
  const row=this.pending.get(message.sequence);if(!row||row.status!=='pending')return false;
  row.status=message.accepted?'accepted':'rejected';row.reasonKey=message.reason_key;
  if([...this.pending.values()].some(r=>r.status==='pending'))return false;
  this.barrier=querySerial;return true;
 }
 setMovement(socket:object,serial:bigint){
  if(socket===this.socket&&this.pending&&this.barrier!==null&&serial>this.barrier){this.pending=null;this.barrier=null;}
 }
}
