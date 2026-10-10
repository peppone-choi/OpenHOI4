import type {ClientMessage,ServerMessage} from './proto/protocol';
type Envelope=Extract<ClientMessage,{type:'Command'|'ProductionCommand'|'EconomyCommand'|'MilitaryCommand'}>;
type Body=Envelope extends infer E?E extends Envelope?Omit<E,'sequence'>:never:never;
/** Shared canonical sequence; at most 64 unacknowledged successful sends. */
export class CommandLedger {
  private socket:object|null=null;
  private serial=0n;
  private issued=new Map<string,Envelope['type']>();
  begin(socket:object){this.socket=socket;this.issued.clear();}
  end(socket:object){if(socket!==this.socket)return;this.socket=null;this.issued.clear();}
  issue(socket:object,body:Body,transmit:(message:Envelope)=>boolean):string|null{
    if(socket!==this.socket||this.issued.size>=64||this.serial===18446744073709551615n)return null;
    const next=this.serial+1n,sequence=next.toString();
    if(!transmit({...body,sequence} as Envelope))return null;
    this.serial=next;this.issued.set(sequence,body.type);return sequence;
  }
  accept(socket:object,message:Extract<ServerMessage,{type:'CommandResult'}>):Envelope['type']|null{
    if(socket!==this.socket)return null;
    const kind=this.issued.get(message.sequence);if(!kind)return null;
    this.issued.delete(message.sequence);return kind;
  }
}
