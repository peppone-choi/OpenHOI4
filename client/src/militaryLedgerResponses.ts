import type {MilitaryNormalLedgerView,MilitaryView,ServerMessage} from './proto/protocol';
import {normalQtyFields} from './militaryValidation';
export type MilitaryLedgerDisplay={ledger:MilitaryNormalLedgerView|null;template:string|null;status:'idle'|'loading'|'ready'|'stale'|'unsupported'|'unavailable';reasonKey:string|null};
export const emptyMilitaryLedger=():MilitaryLedgerDisplay=>({ledger:null,template:null,status:'idle',reasonKey:null});
type Reply=Extract<ServerMessage,{type:'MilitaryNormalLedgerCapabilityResult'|'MilitaryNormalLedgerResult'|'QueryResult'}>;
type Transmit=(request:string,kind:string)=>boolean;
/** Opt-in channel: no game arithmetic, Delta polling, or base-view mutation. */
export class MilitaryLedgerResponses {
  private socket:object|null=null;
  private epoch=0;private generation=0;private serial=0n;
  private capability:'unknown'|'yes'|'no'='unknown';
  private pending:{request:string;epoch:number;generation:number;template:string|null;probe:boolean}|null=null;
  private desired:string|null=null;private dirty=false;
  private base:MilitaryView|null=null;private blockedHash:string|null=null;
  display=emptyMilitaryLedger();
  begin(socket:object){this.close();this.socket=socket;}
  close(){this.epoch++;this.generation++;this.socket=null;this.pending=null;this.desired=null;this.dirty=false;this.base=null;this.blockedHash=null;this.capability='unknown';this.display=emptyMilitaryLedger();}
  end(socket:object){if(socket!==this.socket)return false;const display=this.display;this.close();this.display={...display,status:display.ledger?'stale':'unavailable',reasonKey:display.ledger?null:'military-ledger-disconnected'};return true;}
  select(template:string|null){
    if(template===this.desired&&template===this.display.template)return;
    this.generation++;this.desired=template;this.dirty=template!==null;
    const reasonKey=!template?null:this.socket===null?'military-ledger-disconnected':this.capability==='no'?'unsupported-query':this.blockedHash!==null?'military-ledger-definition-mismatch':null;
    this.display={ledger:null,template,status:!template?'idle':this.capability==='no'?'unsupported':reasonKey?'unavailable':'loading',reasonKey};
  }
  setBase(base:MilitaryView|null){
    const changed=this.base!==null&&this.base.definitions_hash!==base?.definitions_hash;
    this.base=base;
    if(changed){this.generation++;this.blockedHash=null;this.dirty=this.desired!==null;this.display={ledger:null,template:this.desired,status:this.desired?'loading':'idle',reasonKey:null};}
    if(this.desired&&base&&!base.templates.some(t=>t.template===this.desired))this.select(null);
    return this.display;
  }
  issue(socket:object,transmit:Transmit):string|null{
    if(socket!==this.socket||this.pending||!this.desired||this.capability==='no'||this.blockedHash!==null)return null;
    const probe=this.capability==='unknown';
    if(!probe&&!this.dirty)return null;
    if(this.serial===18446744073709551615n)return null;
    const next=this.serial+1n,request=`military-ledger:${next}`;
    const kind=probe?'military-normal-ledger.v1':`military-normal-ledger.v1:${this.desired}`;
    if(!transmit(request,kind))return null;
    this.serial=next;this.pending={request,epoch:this.epoch,generation:this.generation,template:this.desired,probe};
    if(!probe)this.dirty=false;
    return request;
  }
  accept(socket:object,message:Reply):MilitaryLedgerDisplay|null{
    const pending=this.pending;
    if(socket!==this.socket||!pending||pending.epoch!==this.epoch||message.request!==pending.request)return null;
    if(pending.probe){
      if(message.type==='QueryResult'&&!message.supported&&message.reason_key==='unsupported-query'&&message.state===null){
        this.pending=null;this.capability='no';this.dirty=false;
        return this.display={ledger:null,template:this.desired,status:'unsupported',reasonKey:'unsupported-query'};
      }
      if(message.type!=='MilitaryNormalLedgerCapabilityResult')return null;
      this.pending=null;this.capability=message.supported?'yes':'no';this.dirty=message.supported&&this.desired!==null;
      return this.display={ledger:null,template:this.desired,status:message.supported?'loading':'unsupported',reasonKey:message.reason_key};
    }
    if(message.type!=='MilitaryNormalLedgerResult')return null;
    this.pending=null;
    // A -> B -> A must not admit the first A reply.
    if(pending.generation!==this.generation||pending.template!==this.desired)return null;
    if(!message.supported)return this.display={ledger:null,template:this.desired,status:'unavailable',reasonKey:message.reason_key};
    const ledger=message.ledger!,normal=this.base?.templates.find(t=>t.template===ledger.template)?.normal;
    if(!this.base||ledger.definitions_hash!==this.base.definitions_hash||ledger.template!==this.desired||!normal||!ledger.fields.every((f,i)=>f.value.bits===normal[normalQtyFields[i]].bits)){
      this.blockedHash=this.base?.definitions_hash??'';this.dirty=false;
      return this.display={ledger:null,template:this.desired,status:'unavailable',reasonKey:'military-ledger-definition-mismatch'};
    }
    return this.display={ledger,template:this.desired,status:'ready',reasonKey:null};
  }
}
