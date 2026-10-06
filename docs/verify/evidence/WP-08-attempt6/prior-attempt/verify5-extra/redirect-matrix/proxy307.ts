import { createServer, request as httpRequest } from 'node:http';
import { createConnection } from 'node:net';
import { once } from 'node:events';
import type { Duplex } from 'node:stream';

/** Fault-injection HTTP proxy; all normal HTTP and WS bytes come from Rust. */
export async function mapRedirectFixture(upstream: string, options: {
 resource: 'metadata' | 'index'; location: () => string; redirect: () => boolean;
 metadata?: () => unknown | undefined;
}) {
 const target = new URL(upstream), sockets = new Set<Duplex>();
 const sent: { url: string; status: number; location: string }[] = [];
 const proxy = createServer((req, res) => {
  const path = new URL(req.url!, 'http://fixture.invalid');
  const resource = options.resource === 'metadata' ? '/maps/testland/metadata' : '/maps/testland/index.bin';
  if (path.pathname === '/map-fetch-control' || (path.pathname === resource && !path.searchParams.has('alias') && options.redirect())) {
   const location = options.location();
   sent.push({ url: req.url!, status: 307, location });
   res.writeHead(307, { location }); res.end(); return;
  }
  const meta = options.metadata?.();
  if (meta !== undefined && path.pathname === '/maps/testland/metadata') {
   res.writeHead(200, { 'content-type': 'application/json' }); res.end(JSON.stringify(meta)); return;
  }
  const outbound = httpRequest(new URL(req.url!, upstream), { method: req.method, headers: { ...req.headers, host: target.host } }, response => {
   res.writeHead(response.statusCode!, response.headers); response.pipe(res);
  });
  outbound.on('error', error => { if (!res.headersSent) res.writeHead(502); res.end(String(error)); });
  req.pipe(outbound);
 });
 proxy.on('connection', socket => { sockets.add(socket); socket.on('close', () => sockets.delete(socket)); });
 proxy.on('upgrade', (req, socket, head) => {
  const upstreamSocket = createConnection({ host: target.hostname, port: Number(target.port) });
  sockets.add(upstreamSocket); upstreamSocket.on('close', () => sockets.delete(upstreamSocket));
  upstreamSocket.on('connect', () => {
   const headers = req.rawHeaders.reduce<string[]>((lines, value, i, all) => { if (i % 2 === 0) lines.push(`${value}: ${all[i + 1]}`); return lines; }, []).join('\r\n');
   upstreamSocket.write(`${req.method} ${req.url} HTTP/${req.httpVersion}\r\n${headers}\r\n\r\n`);
   if (head.length) upstreamSocket.write(head);
   socket.pipe(upstreamSocket); upstreamSocket.pipe(socket);
  });
  upstreamSocket.on('error', () => socket.destroy()); socket.on('error', () => upstreamSocket.destroy());
  socket.on('close', () => upstreamSocket.destroy());
 });
 proxy.listen(0, '127.0.0.1'); await once(proxy, 'listening');
 const address = proxy.address(); if (!address || typeof address === 'string') throw new Error('source fixture address');
 return { origin: `http://127.0.0.1:${address.port}`, sent,
  close: () => new Promise<void>((resolve, reject) => { proxy.close(e => e ? reject(e) : resolve()); for (const socket of sockets) socket.destroy(); }),
 };
}
