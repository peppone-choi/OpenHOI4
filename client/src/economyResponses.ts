import type { EconomyView, ServerMessage } from './proto/protocol';

export type EconomyDisplay = {
  view: EconomyView | null;
  status: 'loading' | 'ready' | 'stale' | 'unsupported' | 'disconnected';
  reasonKey: string | null;
};

/** Response correlation only. No transport validation or game evaluation. */
export class EconomyResponses {
  private socket: object | null = null;
  private nation: number | null = null;
  private latest: {request: string; nation: number | null} | null = null;
  private dirty = false;
  private serial = 0n;
  begin(socket: object, nation: number | null) {
    this.socket = socket;
    this.nation = nation;
    this.latest = null;
    this.dirty = false;
  }
  issue(socket: object): string | null {
    if (socket !== this.socket) return null;
    if (this.latest) { this.dirty = true; return null; }
    const request = `economy:${++this.serial}`;
    this.latest = {request, nation:this.nation};
    this.dirty = false;
    return request;
  }
  followUp(socket: object): string | null {
    if (socket !== this.socket || this.latest || !this.dirty) return null;
    return this.issue(socket);
  }
  accept(socket: object, message: Extract<ServerMessage, {type:'EconomyResult'|'QueryResult'}>): EconomyDisplay | null {
    // Frozen hosts without economy answer through the existing generic query.
    // A supported time-state reply cannot supply an economic projection.
    if(message.type==='QueryResult'&&(message.supported||message.state!==null))return null;
    if (socket !== this.socket || !this.latest || message.request !== this.latest.request || this.nation !== this.latest.nation) return null;
    this.latest = null;
    return {view:message.type==='EconomyResult'?message.economy:null,status:message.supported ? 'ready' : 'unsupported',reasonKey:message.reason_key};
  }
  end(socket: object): boolean {
    if (socket !== this.socket) return false;
    this.socket = null;
    this.latest = null;
    this.dirty = false;
    return true;
  }
}
