// One-off capability diagnosis: no Rust/game/UI assertions or performance claims.
const {createRequire}=require('node:module');
const {resolve}=require('node:path');
const {mkdirSync,writeFileSync}=require('node:fs');
const http=require('node:http');
const os=require('node:os');
const fromClient=createRequire(resolve(__dirname,'../client/package.json'));
const playwright=fromClient('playwright');
const headed=process.argv.includes('--headed');
const out=resolve(process.env.OH_PROBE_OUT||'target/wp08/linux-probe');
mkdirSync(out,{recursive:true});
(async()=>{
 const server=http.createServer((req,res)=>{res.writeHead(200,{'Content-Type':'text/html'});res.end('<!doctype html><title>WP08 capability probe</title>');});
 await new Promise(done=>server.listen(0,'127.0.0.1',done));
 const url=`http://127.0.0.1:${server.address().port}`;
 let failed=false;
 try{for(const name of ['chromium','firefox','webkit']){
  let browser;
  const report={sourceCommit:process.env.GITHUB_SHA||null,platform:os.platform(),release:os.release(),node:process.version,name,kind:'pure capability probe; no Rust/game assertions',launchOptions:{headless:!headed},customPreferences:null,customGPUFlags:[],sandboxOverrides:[],console:[],pageErrors:[]};
  try{
   browser=await playwright[name].launch({headless:!headed});report.browserVersion=browser.version();
   const page=await browser.newPage({viewport:{width:1280,height:720}});
   page.on('console',m=>report.console.push({type:m.type(),text:m.text()}));page.on('pageerror',e=>report.pageErrors.push(e.message));
   await page.goto(url);
   report.capabilities=await page.evaluate(async()=>{
    const result={userAgent:navigator.userAgent,secureContext:window.isSecureContext,webgl:[],webgpu:{present:!!navigator.gpu}};
    for(const type of ['webgl','webgl2']){
     const canvas=document.createElement('canvas');canvas.width=canvas.height=64;document.body.append(canvas);
     const r={type,creationErrors:[],available:false};canvas.addEventListener('webglcontextcreationerror',e=>r.creationErrors.push(e.statusMessage));
     try{
      const gl=canvas.getContext(type,{antialias:false,alpha:false});r.available=!!gl;
      if(gl){const ext=gl.getExtension('WEBGL_debug_renderer_info');r.version=gl.getParameter(gl.VERSION);r.vendor=gl.getParameter(gl.VENDOR);r.renderer=gl.getParameter(ext?.UNMASKED_RENDERER_WEBGL??gl.RENDERER);r.maxTextureSize=gl.getParameter(gl.MAX_TEXTURE_SIZE);r.extensions=gl.getSupportedExtensions();
       gl.clearColor(37/255,91/255,173/255,1);gl.clear(gl.COLOR_BUFFER_BIT);const rgba=new Uint8Array(4);gl.readPixels(32,32,1,1,gl.RGBA,gl.UNSIGNED_BYTE,rgba);r.pixel=Array.from(rgba);r.glError=gl.getError();r.expectedPixel=[37,91,173,255];r.pixelMatches=r.pixel.every((v,i)=>v===r.expectedPixel[i]);
      }
     }catch(e){r.error=String(e);}result.webgl.push(r);
    }
    if(navigator.gpu){try{const adapter=await navigator.gpu.requestAdapter();result.webgpu.adapterAvailable=!!adapter;if(adapter){const i=adapter.info;result.webgpu.info={vendor:i.vendor,architecture:i.architecture,device:i.device,description:i.description};result.webgpu.maxTextureDimension2D=adapter.limits.maxTextureDimension2D;}}catch(e){result.webgpu.error=String(e);}}
    return result;
   });
   await page.screenshot({path:resolve(out,`${name}-${headed?'headed':'headless'}.png`)});
   if(report.pageErrors.length||report.capabilities.webgl.some(r=>r.available&&!r.pixelMatches))failed=true;
  }catch(e){report.failure=String(e);failed=true;}finally{if(browser)await browser.close();}
  const file=resolve(out,`${name}-${headed?'headed':'headless'}.json`);writeFileSync(file,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
 }}finally{await new Promise(done=>server.close(done));}
 if(failed)process.exitCode=1;
})().catch(e=>{console.error(e);process.exitCode=1;});
