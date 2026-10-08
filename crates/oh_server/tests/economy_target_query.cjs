const {createRequire}=require('node:module');
const req=createRequire(require('node:path').resolve('client/package.json'));
const {encode,decode}=req('@msgpack/msgpack');
const ws=new WebSocket(process.argv[2]);let finished=false;
const timeout=setTimeout(()=>{console.error('positive control query timeout');process.exit(1)},15000);
ws.binaryType='arraybuffer';ws.onopen=()=>{for(const message of [{type:'Hello',protocol_version:'m0-v1'},{type:'Join',session:'local',nation:'NTH'},{type:'Command',sequence:'999',command:{type:'Pause',paused:true}},{type:'Query',request:'valid-target2',kind:'economy'}])ws.send(encode(message));};
ws.onmessage=e=>{const value=decode(new Uint8Array(e.data));if(value.type==='EconomyResult'&&value.request==='valid-target2'){if(!value.supported||!value.economy.nations.some(n=>n.projects.some(p=>p.target==='2'))){console.error('valid control target2 missing');process.exit(1)}finished=true;console.log(JSON.stringify({pid:process.pid,result:value}));clearTimeout(timeout);ws.close();}};
ws.onerror=e=>{console.error(String(e));process.exit(1)};ws.onclose=()=>{if(!finished)process.exit(1)};
