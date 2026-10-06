import { Color, DataTexture, Mesh, MeshBasicNodeMaterial, NearestFilter, NoColorSpace, NoToneMapping, LinearSRGBColorSpace, OrthographicCamera, PlaneGeometry, RGFormat, RGBAFormat, Scene, UnsignedByteType, Vector2, WebGPURenderer } from 'three/webgpu';
import { Fn, float, vec2, vec3, vec4, textureLoad, uniform, uv, mix } from 'three/tsl';
import type Node from 'three/src/nodes/core/Node.js';
import type { PixelFormat } from 'three';
import type { MapMetadata, WorldView } from '../proto/protocol';
import { LOOKUP_SIZE, palettes, pick, updateColors } from './model';

function dataTexture(bytes:Uint8Array,width:number,height:number,format:PixelFormat=RGBAFormat){
 const t=new DataTexture(bytes,width,height,format,UnsignedByteType);
 t.minFilter=t.magFilter=NearestFilter;t.generateMipmaps=false;t.flipY=false;t.colorSpace=NoColorSpace;t.needsUpdate=true;return t;
}
export async function createMap(host:HTMLDivElement,meta:MapMetadata,bytes:Uint8Array,index:Uint16Array,initial:WorldView,onHover:(id:number|null)=>void,onPick:(id:number|null)=>void,onFailure:()=>void,forceWebGL=false){
 const {width,height,style}=meta;
 const renderer=new WebGPURenderer({forceWebGL,antialias:false,alpha:false});
 let disposed=false;
 try{await renderer.init();}catch(e){renderer.dispose();throw e;}
 const backend=renderer.backend as typeof renderer.backend & {isWebGPUBackend?:boolean;device?:GPUDevice;gl?:WebGL2RenderingContext};
 const limit=backend.device?.limits.maxTextureDimension2D??backend.gl?.getParameter(backend.gl.MAX_TEXTURE_SIZE)??0;
 if(Math.max(width,height,LOOKUP_SIZE)>limit){renderer.dispose();throw new Error('map-texture-limit');}
 host.replaceChildren(renderer.domElement);host.dataset.backend=backend.isWebGPUBackend?'webgpu':'webgl2';host.dataset.textureLimit=String(limit);
 if(backend.device){host.dataset.adapter=JSON.stringify(backend.device.adapterInfo);host.dataset.device=backend.device.label||'WebGPU device';}
 if(backend.gl){const ext=backend.gl.getExtension('WEBGL_debug_renderer_info');host.dataset.adapter=String(backend.gl.getParameter(ext?.UNMASKED_RENDERER_WEBGL??backend.gl.RENDERER));host.dataset.device=String(backend.gl.getParameter(backend.gl.VERSION));}
 renderer.onDeviceLost=()=>{if(!disposed)onFailure();};
 renderer.outputColorSpace=LinearSRGBColorSpace;renderer.toneMapping=NoToneMapping;
 const rgb=(c:number[])=>vec3(c[0]/255,c[1]/255,c[2]/255);
 const arrays=palettes(initial,'map-mode-owner');
 const indexTexture=dataTexture(bytes,width,height,RGFormat),colors=dataTexture(arrays.colors,LOOKUP_SIZE,LOOKUP_SIZE),nations=dataTexture(arrays.nations,LOOKUP_SIZE,LOOKUP_SIZE),states=dataTexture(arrays.states,LOOKUP_SIZE,LOOKUP_SIZE);
 const selected=uniform(-1),hovered=uniform(-1),units=uniform(1);
 const texel=Fn(([point]:[Node<'vec2'>])=>{
  const p=point.floor().clamp(vec2(0),vec2(width-1,height-1));const rg=textureLoad(indexTexture,p).rg.mul(255).round();return rg.x.add(rg.y.mul(256));
 });
 const lookup=Fn(([id]:[Node<'float'>])=>vec2(id.mod(256),id.div(256).floor()));
 const material=new MeshBasicNodeMaterial();material.toneMapped=false;
 material.colorNode=Fn(()=>{
  const point=vec2(uv().x,uv().y.oneMinus()).mul(vec2(width,height));const id=texel(point);
  const address=lookup(id);const base=textureLoad(colors,address).rgb.toVar();
  const boundary=(ref:DataTexture|null,thickness:number)=>{
   const step=units.mul(thickness/1000);
   const own=ref?textureLoad(ref,address).rgb:vec3(id,0,0);
   const offsets=[vec2(step,0),vec2(step.negate(),0),vec2(0,step),vec2(0,step.negate())];
   let edge:Node<'float'>=float(0);
   for(const offset of offsets){const other=texel(point.add(offset));const value=ref?textureLoad(ref,lookup(other)).rgb:vec3(other,0,0);edge=edge.max(own.sub(value).abs().dot(vec3(1)).greaterThan(0).toFloat());}
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
 let viewWidth=width,viewHeight=height,zoom=1,frame=0,started=performance.now(),mode='map-mode-owner',world=initial;
 const publishCamera=()=>{units.value=viewWidth/host.clientWidth/zoom;host.dataset.camera=JSON.stringify({width:viewWidth/zoom,height:viewHeight/zoom,x:camera.position.x+width/2,y:height/2-camera.position.y,zoom});};
 const resize=()=>{
  const w=host.clientWidth,h=host.clientHeight;if(!w||!h)return;
  renderer.setPixelRatio(window.devicePixelRatio);renderer.setSize(w,h);
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
 const wheel=(e:WheelEvent)=>{e.preventDefault();const before=coordinates(e);zoom=Math.min(style.zoom_max_milli/1000,Math.max(style.zoom_min_milli/1000,zoom*Math.exp(-e.deltaY*style.wheel_milli/100000)));camera.zoom=zoom;camera.updateProjectionMatrix();const after=coordinates(e);camera.position.x+=before.x-after.x;camera.position.y-=before.y-after.y;publishCamera();};
 host.addEventListener('pointerdown',down);host.addEventListener('pointermove',move);host.addEventListener('pointerup',up);host.addEventListener('pointercancel',leave);host.addEventListener('pointerleave',leave);host.addEventListener('wheel',wheel,{passive:false});
 try{await renderer.compileAsync(scene,camera);renderer.render(scene,camera);}catch(e){observer.disconnect();renderer.dispose();geometry.dispose();material.dispose();for(const t of [indexTexture,colors,nations,states])t.dispose();host.replaceChildren();throw e;}
 host.dataset.frames='1';
 let raf=0;
 const animate=()=>{if(disposed)return;try{renderer.render(scene,camera);host.dataset.frames=String(++frame);host.dataset.frameMs=String((performance.now()-started)/frame);raf=requestAnimationFrame(animate);}catch{onFailure();}};raf=requestAnimationFrame(animate);
 return {
  update(next:WorldView,nextMode:string,id:number|null){
   world=next;mode=nextMode;selected.value=id===null?-1:world.province_ids.indexOf(id);host.dataset.selected=String(id??'');host.dataset.mode=mode;
   const began=performance.now();const ranges=updateColors(arrays.colors,world,mode);
   // Upload only changed texels through Three's public cross-backend copy API.
   for(const range of ranges){const dense=range.start/4;const patch=dataTexture(arrays.colors.slice(range.start,range.start+range.count),1,1);renderer.copyTextureToTexture(patch,colors,null,new Vector2(dense%256,Math.floor(dense/256)));patch.dispose();}
   const refs=palettes(world,mode);
   for(const [data,next,texture] of [[arrays.nations,refs.nations,nations],[arrays.states,refs.states,states]] as const){
    world.province_ids.forEach((_,i)=>{const start=i*4;if(next.subarray(start,start+4).some((v,j)=>data[start+j]!==v)){data.set(next.subarray(start,start+4),start);const patch=dataTexture(data.slice(start,start+4),1,1);renderer.copyTextureToTexture(patch,texture,null,new Vector2(i%256,Math.floor(i/256)));patch.dispose();}});
   }
   host.dataset.updatedBytes=String(ranges.reduce((n,r)=>n+r.count,0));host.dataset.updateMs=String(performance.now()-began);
  },
  reset(){zoom=1;camera.position.x=camera.position.y=0;resize();},
  dispose(){disposed=true;cancelAnimationFrame(raf);observer.disconnect();host.removeEventListener('pointerdown',down);host.removeEventListener('pointermove',move);host.removeEventListener('pointerup',up);host.removeEventListener('pointercancel',leave);host.removeEventListener('pointerleave',leave);host.removeEventListener('wheel',wheel);renderer.dispose();geometry.dispose();material.dispose();for(const t of [indexTexture,colors,nations,states])t.dispose();host.replaceChildren();},
 };
}
