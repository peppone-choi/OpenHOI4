import { decode, encode } from "file:///E:/openhoi/.orchestrator/wt/WP-12-verify2/client/node_modules/@msgpack/msgpack/dist.esm/index.mjs";
import { PROTOCOL_VERSION } from "file:///E:/openhoi/.orchestrator/wt/WP-12-verify2/client/src/proto/protocol.ts";
const string = (value)=>typeof value === 'string';
const boolean = (value)=>typeof value === 'boolean';
const u8 = (value)=>typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 0xff;
const literal = (expected)=>(value)=>value === expected;
const nullable = (guard)=>(value)=>value === null || guard(value);
const array = (guard)=>(value)=>Array.isArray(value) && value.every(guard);
function shape(fields) {
    return (value)=>typeof value === 'object' && value !== null && !Array.isArray(value) && Object.entries(fields).every(([key, guard])=>guard(value[key]));
}
const timeState = shape({
    date: string,
    hour: u8,
    tick: string,
    paused: boolean,
    speed: u8
});
const packInfo = shape({
    id: string,
    version: string,
    hash: string
});
const serverFields = {
    Welcome: {
        type: literal('Welcome'),
        engine_version: string,
        protocol_version: string,
        accepted: boolean,
        reason_key: nullable(string),
        packs: array(packInfo),
        sessions: array(string)
    },
    CommandResult: {
        type: literal('CommandResult'),
        sequence: string,
        accepted: boolean,
        reason_key: nullable(string)
    },
    Snapshot: {
        type: literal('Snapshot'),
        state: timeState
    },
    Delta: {
        type: literal('Delta'),
        sequence: string,
        state: timeState
    },
    QueryResult: {
        type: literal('QueryResult'),
        request: string,
        supported: boolean,
        reason_key: nullable(string),
        state: nullable(timeState)
    },
    Notice: {
        type: literal('Notice'),
        key: string
    }
};
const serverGuards = Object.values(serverFields).map((fields)=>shape(fields));
export function isServerMessage(value) {
    return serverGuards.some((guard)=>guard(value));
}
export function applyServerMessage(current, message) {
    return message.type === 'Snapshot' || message.type === 'Delta' ? message.state : current;
}
export function connect(onMessage, onClose) {
    const ws = new WebSocket(`${location.protocol === 'https:' ? 'wss' : 'ws'}://${location.host}/ws`);
    ws.binaryType = 'arraybuffer';
    let closed = false;
    const detach = ()=>{
        ws.onopen = null;
        ws.onmessage = null;
        ws.onclose = null;
        ws.onerror = null;
    };
    const disconnect = (reasonKey)=>{
        if (closed) return;
        closed = true;
        detach();
        ws.close();
        onClose(reasonKey);
    };
    const send = (message)=>{
        if (!closed && ws.readyState === WebSocket.OPEN) ws.send(encode(message));
    };
    ws.onopen = ()=>send({
            type: 'Hello',
            protocol_version: PROTOCOL_VERSION
        });
    ws.onmessage = (event)=>{
        try {
            if (!(event.data instanceof ArrayBuffer)) throw new Error('Expected binary server frame');
            const message = decode(new Uint8Array(event.data));
            if (!isServerMessage(message)) throw new Error('Invalid server message shape');
            onMessage(message);
            if (message.type === 'Welcome' && message.accepted) send({
                type: 'Join',
                session: 'local',
                nation: null
            });
        } catch  {
            disconnect('invalid-server-message');
        }
    };
    ws.onclose = ()=>disconnect();
    ws.onerror = ()=>disconnect();
    return {
        send,
        close: ()=>{
            closed = true;
            detach();
            ws.close();
        }
    };
}
