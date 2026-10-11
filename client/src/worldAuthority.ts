import type {ServerMessage,WorldView} from './proto/protocol';
import {validateWorldDisplay} from './map/model';

/** State membership is the existing loader/world land contract, never terrain or owner. */
export function landSet(world:WorldView):Set<number>|null{
  try{validateWorldDisplay(world);}catch{return null;}
  const land=new Set<number>();
  for(const state of world.states)for(const id of state.provinces){
    if(land.has(id)||world.provinces.find(p=>p.id===id)?.state!==state.id)return null;
    land.add(id);
  }
  return land;
}
export type DeploymentWorld={world:WorldView;land:Set<number>;serial:bigint};
/** Dedicated full-world correlation. Retained map display never authorizes a command. */
export class WorldAuthority {
  private socket:object|null=null;
  private epoch=0;
  private serial=0n;
  private pending:{request:string;epoch:number;serial:bigint}|null=null;
  private dirty=false;
  private current:DeploymentWorld|null=null;
  private revision=0;
  get issuedSerial(){return this.serial;}
  get authority(){return this.current;}
  get generation(){return this.revision;}
  revoke(){this.current=null;this.revision++;}
  begin(socket:object){this.close();this.socket=socket;}
  close(){this.epoch++;this.socket=null;this.pending=null;this.dirty=false;this.revoke();}
  end(socket:object){if(socket!==this.socket)return false;this.close();return true;}
  issue(socket:object,transmit:(request:string)=>boolean):string|null{
    if(socket!==this.socket)return null;
    this.revoke();
    if(this.pending){this.dirty=true;return null;}
    const serial=this.serial+1n,request=`deploy-world:${serial}`;
    if(!transmit(request))return null;
    this.serial=serial;this.pending={request,epoch:this.epoch,serial};this.dirty=false;return request;
  }
  followUp(socket:object,transmit:(request:string)=>boolean):string|null{
    if(socket!==this.socket||this.pending||!this.dirty)return null;
    return this.issue(socket,transmit);
  }
  accept(socket:object,message:Extract<ServerMessage,{type:'WorldResult'|'QueryResult'}>):boolean{
    if(message.type==='QueryResult'&&(message.supported||message.state!==null))return false;
    if(socket!==this.socket||!this.pending||this.pending.epoch!==this.epoch||message.request!==this.pending.request)return false;
    const {serial}=this.pending;this.pending=null;this.current=null;
    if(message.type==='WorldResult'&&message.supported&&message.world){
      const land=landSet(message.world);if(land)this.current={world:message.world,land,serial};
    }
    return true;
  }
  candidates(nation:number):number[]{return this.current?.world.provinces.filter(p=>this.current!.land.has(p.id)&&p.controller===nation).map(p=>p.id)??[];}
}
