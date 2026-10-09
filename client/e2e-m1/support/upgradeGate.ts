import {createServer,request as httpRequest,type IncomingMessage} from 'node:http';
import {connect,type Socket} from 'node:net';

/** Holds the real HTTP upgrade before OPEN; forwards all actual Rust frames. */
export async function upgradeGate(upstream:string) {
  const target=new URL(upstream),sockets=new Set<Socket>();
  let hold=false;
  const pending:Array<()=>void>=[];
  const server=createServer((req,res)=>{
    const forwarded=httpRequest(new URL(req.url??'/',target),{method:req.method,headers:req.headers},reply=>{
      res.writeHead(reply.statusCode??502,reply.headers);reply.pipe(res);
    });
    forwarded.on('error',()=>{res.writeHead(502);res.end();});req.pipe(forwarded);
  });
  server.on('connection',socket=>{sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
  server.on('upgrade',(req:IncomingMessage,socket:Socket,head:Buffer)=>{
    const forward=()=>{
      if(socket.destroyed)return;
      const backend=connect(Number(target.port),target.hostname,()=>{
        backend.write(`${req.method} ${req.url} HTTP/${req.httpVersion}\r\n${req.rawHeaders.reduce((headers,value,index)=>headers+(index%2===0?`${value}: `:`${value}\r\n`),'')}\r\n`);
        if(head.length)backend.write(head);
        socket.pipe(backend);backend.pipe(socket);
      });
      sockets.add(backend);backend.on('close',()=>sockets.delete(backend));
      backend.on('error',()=>socket.destroy());socket.on('error',()=>backend.destroy());
      socket.on('close',()=>backend.destroy());
    };
    if(hold)pending.push(forward);else forward();
  });
  await new Promise<void>(resolve=>server.listen(0,'127.0.0.1',resolve));
  const address=server.address();if(!address||typeof address==='string')throw new Error('upgrade gate address');
  return {
    url:`http://127.0.0.1:${address.port}`,
    hold:()=>{hold=true;},
    count:()=>pending.length,
    release:()=>{hold=false;for(const forward of pending.splice(0))forward();},
    close:async()=>{for(const socket of sockets)socket.destroy();await new Promise<void>(resolve=>server.close(()=>resolve()));},
  };
}
