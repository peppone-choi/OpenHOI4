import { Color, DataTexture, Mesh, MeshBasicNodeMaterial, NearestFilter, NoColorSpace, NoToneMapping, LinearSRGBColorSpace, OrthographicCamera, PlaneGeometry, RGFormat, RGBAFormat, Scene, UnsignedByteType, Vector2, WebGPURenderer } from 'three/webgpu';
import { Fn, float, vec2, vec3, vec4, textureLoad, uniform, uv, mix } from 'three/tsl';
import type Node from 'three/src/nodes/core/Node.js';
import type { PixelFormat } from 'three';
import type { MapMetadata, WorldView } from '../proto/protocol';
import { LOOKUP_SIZE, palettes, pick, updateColors,validateWorldDisplay } from './model';

function dataTexture(bytes:Uint8Array,width:number,height:number,format:PixelFormat=RGBAFormat){
 const t=new DataTexture(bytes,width,height,format,UnsignedByteType);
 t.minFilter=t.magFilter=NearestFilter;t.generateMipmaps=false;t.flipY=false;t.colorSpace=NoColorSpace;t.needsUpdate=true;return t;
}
export type MapCamera={x:number;y:number;zoom:number};
export async function createMap(host:HTMLDivElement,meta:MapMetadata,bytes:Uint8Array,index:Uint16Array,initial:WorldView,onHover:(id:number|null)=>void,onPick:(id:number|null)=>void,onFailure:()=>void,onResize:(camera:MapCamera)=>void,forceWebGL=false,presentation?:{camera?:MapCamera;mode:string;selected:number|null;signal?:AbortSignal;sceneExtension?:(scene:Scene,camera:OrthographicCamera)=>Promise<()=>void>;paletteAdapter?:{palettes:typeof palettes;updateColors:typeof updateColors};coastKinds?:Uint8Array}){
 const {width,height,style}=meta;
 const renderer=new WebGPURenderer({forceWebGL,antialias:false,alpha:false});
 let disposed=false;
 try{await renderer.init();}catch(e){
  // Three r186 dispose() starts setAnimationLoop(null), which reuses the
  // rejected init promise. Release any partial GPU allocation without
  // re-entering that failed initialization; the unattached canvas can be GC'd.
  const partial=renderer.backend as typeof renderer.backend & {device?:GPUDevice;gl?:WebGL2RenderingContext};
  partial.device?.destroy();partial.gl?.getExtension('WEBGL_lose_context')?.loseContext();
  throw e;
 }
 if(presentation?.signal?.aborted){disposed=true;await renderer.dispose();throw new Error('map-aborted');}
 const clearCanvas=()=>{if(renderer.domElement.parentElement===host){renderer.domElement.remove();delete host.dataset.backend;delete host.dataset.frames;}};
 const backend=renderer.backend as typeof renderer.backend & {isWebGPUBackend?:boolean;device?:GPUDevice;gl?:WebGL2RenderingContext};
 const limit=backend.device?.limits.maxTextureDimension2D??backend.gl?.getParameter(backend.gl.MAX_TEXTURE_SIZE)??0;
 if(Math.max(width,height,LOOKUP_SIZE)>limit){renderer.dispose();throw new Error('map-texture-limit');}
 host.replaceChildren(renderer.domElement);host.dataset.backend=backend.isWebGPUBackend?'webgpu':'webgl2';host.dataset.textureLimit=String(limit);
 if(backend.device){const info=backend.device.adapterInfo;host.dataset.adapter=JSON.stringify({vendor:info.vendor,architecture:info.architecture,device:info.device,description:info.description});host.dataset.device=backend.device.label||'WebGPU device';}
 if(backend.gl){const ext=backend.gl.getExtension('WEBGL_debug_renderer_info');host.dataset.adapter=String(backend.gl.getParameter(ext?.UNMASKED_RENDERER_WEBGL??backend.gl.RENDERER));host.dataset.device=String(backend.gl.getParameter(backend.gl.VERSION));}
 renderer.onDeviceLost=()=>{if(!disposed)onFailure();};
 // r186 resolves compileAsync even when native pipeline creation failed.
 // Observe its per-backend pipeline records without changing compilation.
 const pipelineBackend=backend as typeof backend & {createRenderPipeline(object:{pipeline:object},promises:Promise<unknown>[]|null):void;get(key:object):{error?:boolean;programGPU?:WebGLProgram}};
 const pipelines=new Set<object>(),linked=new WeakSet<object>(),createPipeline=pipelineBackend.createRenderPipeline;
 pipelineBackend.createRenderPipeline=function(object,promises){createPipeline.call(this,object,promises);pipelines.add(object.pipeline);};
 const assertPipelines=()=>{for(const pipeline of pipelines){
  const data=pipelineBackend.get(pipeline);if(data.error)throw new Error('map-pipeline-error');
  if(backend.gl&&data.programGPU&&!linked.has(pipeline)){if(!backend.gl.getProgramParameter(data.programGPU,backend.gl.LINK_STATUS))throw new Error('map-pipeline-error');linked.add(pipeline);}
 }};
 renderer.outputColorSpace=LinearSRGBColorSpace;renderer.toneMapping=NoToneMapping;
 const rgb=(c:number[])=>vec3(c[0]/255,c[1]/255,c[2]/255);
 const paletteFor=presentation?.paletteAdapter?.palettes.bind(presentation.paletteAdapter)??palettes;
 const updatePalette=presentation?.paletteAdapter?.updateColors.bind(presentation.paletteAdapter)??updateColors;
 const arrays=paletteFor(initial,presentation?.mode??'map-mode-owner');
 const indexTexture=dataTexture(bytes,width,height,RGFormat),colors=dataTexture(arrays.colors,LOOKUP_SIZE,LOOKUP_SIZE),nations=dataTexture(arrays.nations,LOOKUP_SIZE,LOOKUP_SIZE),states=dataTexture(arrays.states,LOOKUP_SIZE,LOOKUP_SIZE);
 const coastKinds=presentation?.coastKinds?dataTexture(presentation.coastKinds,LOOKUP_SIZE,LOOKUP_SIZE):null;
 const selected=uniform(presentation?.selected==null?-1:initial.province_ids.indexOf(presentation.selected)),hovered=uniform(-1),units=uniform(1);
 const texel=Fn(([point]:[Node<'vec2'>])=>{
  const p=point.floor().clamp(vec2(0),vec2(width-1,height-1));const rg=textureLoad(indexTexture,p).rg.mul(255).round();return rg.x.add(rg.y.mul(256));
 });
 const lookup=Fn(([id]:[Node<'float'>])=>vec2(id.mod(256),id.div(256).floor()));
 const material=new MeshBasicNodeMaterial();material.toneMapped=false;
 material.colorNode=Fn(()=>{
  const point=vec2(uv().x,uv().y.oneMinus()).mul(vec2(width,height));const id=texel(point);
  const address=lookup(id);const base=textureLoad(colors,address).rgb.toVar();
  if(coastKinds){
   // Colour interpolation only at source land/water transitions. IDs, picking
   // and same-kind province borders always retain the exact integer index.
   const cell=point.sub(vec2(.5)).floor(),fraction=point.sub(vec2(.5)).fract();
   const samples=[cell,cell.add(vec2(1,0)),cell.add(vec2(0,1)),cell.add(vec2(1,1))];
   const refs=samples.map(p=>lookup(texel(p))),kind=textureLoad(coastKinds,address).r;
   let changed:Node<'float'>=float(0);for(const p of refs)changed=changed.max(textureLoad(coastKinds,p).r.sub(kind).abs().greaterThan(0).toFloat());
   const north=mix(textureLoad(colors,refs[0]).rgb,textureLoad(colors,refs[1]).rgb,fraction.x);
   const south=mix(textureLoad(colors,refs[2]).rgb,textureLoad(colors,refs[3]).rgb,fraction.x);
   base.assign(mix(base,mix(north,south,fraction.y),changed));
  }
  const boundary=(ref:DataTexture|null,thickness:number)=>{
   const step=units.mul(thickness/1000);
   const own=ref?textureLoad(ref,address).rgb:vec3(id,0,0);
   const offsets=[vec2(step,0),vec2(step.negate(),0),vec2(0,step),vec2(0,step.negate())];
   let edge:Node<'float'>=float(0);
   for(const offset of offsets){const other=texel(point.add(offset));const value=ref?textureLoad(ref,lookup(other)).rgb:vec3(other,0,0);let different:Node<'float'>=own.sub(value).abs().dot(vec3(1)).greaterThan(0).toFloat();if(!ref&&coastKinds)different=different.mul(textureLoad(coastKinds,address).r.equal(textureLoad(coastKinds,lookup(other)).r).toFloat());edge=edge.max(different);}
   return edge;
  };
  base.assign(mix(base,rgb(style.province_border),boundary(null,style.province_width_milli)));
  base.assign(mix(base,rgb(style.state_border),boundary(states,style.state_width_milli)));
  base.assign(mix(base,rgb(style.nation_border),boundary(nations,style.nation_width_milli)));
  base.assign(mix(base,rgb(style.hovered),id.equal(hovered).toFloat().mul(style.highlight_milli/1000)));
  base.assign(mix(base,rgb(style.selected),id.equal(selected).toFloat().mul(style.highlight_milli/1000)));
  return vec4(base,1);
 })();
 const scene=new Scene();scene.background=new Color().setRGB(style.background[0]/255,style.background[1]/255,style.background[2]/255,LinearSRGBColorSpace);
 const geometry=new PlaneGeometry(width,height);scene.add(new Mesh(geometry,material));
 const camera=new OrthographicCamera();camera.position.z=1;
 camera.position.x=presentation?.camera?.x??0;camera.position.y=presentation?.camera?.y??0;
 let viewWidth=width,viewHeight=height,zoom=presentation?.camera?.zoom??1,frame=0,started=performance.now(),mode=presentation?.mode??'map-mode-owner',world=initial,presented=false,lastWidth=0,lastHeight=0,lastRatio=0,recreateRaf=0;
 const publishCamera=()=>{units.value=viewWidth/host.clientWidth/zoom;if(presented)host.dataset.camera=JSON.stringify({width:viewWidth/zoom,height:viewHeight/zoom,x:camera.position.x+width/2,y:height/2-camera.position.y,zoom});};
 const resize=()=>{
  if(disposed||recreateRaf)return;
  const w=host.clientWidth,h=host.clientHeight;if(!w||!h)return;
  const ratio=window.devicePixelRatio,changed=w!==lastWidth||h!==lastHeight||ratio!==lastRatio;
  // Fresh WebGL presentation surfaces avoid the Win-WebKit resized canvas
  // becoming transparent while its default framebuffer remains correct.
  if(changed&&presented&&backend.gl){observer.disconnect();const saved={x:camera.position.x,y:camera.position.y,zoom};recreateRaf=requestAnimationFrame(()=>{if(!disposed)onResize(saved);});return;}
  if(changed){renderer.setPixelRatio(ratio);renderer.setSize(w,h);lastWidth=w;lastHeight=h;lastRatio=ratio;}
  const aspect=w/h;viewHeight=Math.max(height,width/aspect)/(style.fit_milli/1000);viewWidth=viewHeight*aspect;
  camera.left=-viewWidth/2;camera.right=viewWidth/2;camera.top=viewHeight/2;camera.bottom=-viewHeight/2;camera.zoom=zoom;camera.updateProjectionMatrix();publishCamera();
 };
 const observer=new ResizeObserver(resize);observer.observe(host);resize();
 const coordinates=(e:{clientX:number;clientY:number})=>{const rect=host.getBoundingClientRect();return {x:(e.clientX-rect.left)/rect.width*viewWidth/zoom-viewWidth/zoom/2+camera.position.x+width/2,y:(e.clientY-rect.top)/rect.height*viewHeight/zoom-viewHeight/zoom/2+height/2-camera.position.y};};
 const hover=(e:PointerEvent)=>{const p=coordinates(e);const id=pick(index,world.province_ids,width,height,p.x,p.y);hovered.value=id===null?-1:world.province_ids.indexOf(id);host.dataset.hover=String(id??'');onHover(id);};
 let drag:{x:number;y:number;cx:number;cy:number;moved:boolean}|null=null;
 const down=(e:PointerEvent)=>{if(e.button!==0)return;drag={x:e.clientX,y:e.clientY,cx:camera.position.x,cy:camera.position.y,moved:false};host.setPointerCapture(e.pointerId);};
 const move=(e:PointerEvent)=>{if(drag){const dx=e.clientX-drag.x,dy=e.clientY-drag.y;drag.moved||=Math.hypot(dx,dy)>style.drag_threshold;if(drag.moved){camera.position.x=drag.cx-dx*viewWidth/host.clientWidth/zoom;camera.position.y=drag.cy+dy*viewHeight/host.clientHeight/zoom;publishCamera();}}hover(e);};
 const up=(e:PointerEvent)=>{if(!drag)return;if(!drag.moved){const p=coordinates(e);const id=pick(index,world.province_ids,width,height,p.x,p.y);onPick(id);}drag=null;host.releasePointerCapture(e.pointerId);};
 const leave=()=>{hovered.value=-1;host.dataset.hover='';onHover(null);};
 const cancel=()=>{drag=null;leave();};
 const wheel=(e:WheelEvent)=>{e.preventDefault();const before=coordinates(e);zoom=Math.min(style.zoom_max_milli/1000,Math.max(style.zoom_min_milli/1000,zoom*Math.exp(-e.deltaY*style.wheel_milli/100000)));camera.zoom=zoom;camera.updateProjectionMatrix();const after=coordinates(e);camera.position.x+=before.x-after.x;camera.position.y-=before.y-after.y;publishCamera();};
 let disposeSceneExtension=()=>{};
 try{
  if(presentation?.sceneExtension)disposeSceneExtension=await presentation.sceneExtension(scene,camera);
  await renderer.compileAsync(scene,camera);assertPipelines();
  const shader=await renderer.debug.getShaderAsync(scene,camera,scene.children[0]);
  if(presentation?.signal?.aborted)throw new Error('map-aborted');assertPipelines();
  resize();renderer.render(scene,camera);presented=true;publishCamera();
  host.dataset.shaderLanguage=shader.fragmentShader?.includes('@fragment')?'wgsl':'glsl';host.dataset.shaderLength=String(shader.fragmentShader?.length??0);
 }catch(e){disposed=true;observer.disconnect();disposeSceneExtension();await renderer.dispose();geometry.dispose();material.dispose();coastKinds?.dispose();for(const t of [indexTexture,colors,nations,states])t.dispose();clearCanvas();throw e;}
 host.addEventListener('pointerdown',down);host.addEventListener('pointermove',move);host.addEventListener('pointerup',up);host.addEventListener('pointercancel',cancel);host.addEventListener('pointerleave',leave);host.addEventListener('wheel',wheel,{passive:false});
 host.dataset.frames='1';
 let raf=0;
 // CSS ResizeObserver does not signal a DPR-only change. The existing render
 // loop also checks DPR; it needs no extra timer or browser-specific event.
 const animate=()=>{if(disposed)return;try{if(window.devicePixelRatio!==lastRatio)resize();if(recreateRaf)return;assertPipelines();renderer.render(scene,camera);host.dataset.frames=String(++frame);host.dataset.frameMs=String((performance.now()-started)/frame);raf=requestAnimationFrame(animate);}catch{onFailure();}};raf=requestAnimationFrame(animate);
 return {
  update(next:WorldView,nextMode:string,id:number|null){
   validateWorldDisplay(next);
   world=next;mode=nextMode;selected.value=id===null?-1:world.province_ids.indexOf(id);host.dataset.selected=String(id??'');host.dataset.mode=mode;
   const began=performance.now();const ranges=updatePalette(arrays.colors,world,mode);
   // Upload only changed texels through Three's public cross-backend copy API.
   for(const range of ranges){const dense=range.start/4;const patch=dataTexture(arrays.colors.slice(range.start,range.start+range.count),1,1);renderer.copyTextureToTexture(patch,colors,null,new Vector2(dense%256,Math.floor(dense/256)));patch.dispose();}
   const refs=paletteFor(world,mode);
   for(const [data,next,texture] of [[arrays.nations,refs.nations,nations],[arrays.states,refs.states,states]] as const){
    world.province_ids.forEach((_,i)=>{const start=i*4;if(next.subarray(start,start+4).some((v,j)=>data[start+j]!==v)){data.set(next.subarray(start,start+4),start);const patch=dataTexture(data.slice(start,start+4),1,1);renderer.copyTextureToTexture(patch,texture,null,new Vector2(i%256,Math.floor(i/256)));patch.dispose();}});
   }
   host.dataset.updatedBytes=String(ranges.reduce((n,r)=>n+r.count,0));host.dataset.updateMs=String(performance.now()-began);
   if(ranges.length){host.dataset.lastChangedBytes=host.dataset.updatedBytes;host.dataset.lastUpdateMs=host.dataset.updateMs;host.dataset.totalChangedBytes=String(Number(host.dataset.totalChangedBytes??0)+Number(host.dataset.updatedBytes));}
  },
  reset(){zoom=1;camera.position.x=camera.position.y=0;resize();},
  dispose(){disposed=true;cancelAnimationFrame(raf);cancelAnimationFrame(recreateRaf);observer.disconnect();host.removeEventListener('pointerdown',down);host.removeEventListener('pointermove',move);host.removeEventListener('pointerup',up);host.removeEventListener('pointercancel',cancel);host.removeEventListener('pointerleave',leave);host.removeEventListener('wheel',wheel);disposeSceneExtension();renderer.dispose();geometry.dispose();material.dispose();coastKinds?.dispose();for(const t of [indexTexture,colors,nations,states])t.dispose();clearCanvas();},
 };
}
