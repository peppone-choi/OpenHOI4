import { decode, encode } from '@msgpack/msgpack';
import { PROTOCOL_VERSION, type ClientMessage, type ServerMessage, type TimeState } from './proto/protocol';
export function applyServerMessage(current: TimeState | null, message: ServerMessage): TimeState | null {
  return message.type === 'Snapshot' || message.type === 'Delta' ? message.state : current;
}
export function connect(onMessage: (message: ServerMessage) => void, onClose: () => void) {
  const ws = new WebSocket(`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/ws`);
  ws.binaryType = 'arraybuffer';
  const send = (message: ClientMessage) => { if (ws.readyState === WebSocket.OPEN) ws.send(encode(message)); };
  ws.onopen = () => send({ type: 'Hello', protocol_version: PROTOCOL_VERSION });
  ws.onmessage = event => {
    try {
      const message = decode(new Uint8Array(event.data)) as ServerMessage;
      onMessage(message);
      if (message.type === 'Welcome' && message.accepted) send({ type: 'Join', session: 'local', nation: null });
    } catch { ws.close(); onClose(); }
  };
  ws.onclose = onClose;
  ws.onerror = onClose;
  return { send, close: () => {
    ws.onopen = null; ws.onmessage = null; ws.onclose = null; ws.onerror = null;
    ws.close();
  } };
}
