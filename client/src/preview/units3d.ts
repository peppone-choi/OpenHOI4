import { AmbientLight,DirectionalLight,Group,Mesh,Box3,type Scene,type OrthographicCamera,type BufferGeometry,type Material } from 'three/webgpu';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { sha256 } from './model';

export type UnitSample={asset:'army'|'air'|'navy';longitude:number;latitude:number;heading_degrees:number;scale:number;height:number;tilt_degrees:number};
export type UnitAsset={id:string;file:string;sha256:string};
type GlbJson={asset?:{version?:string};buffers?:{uri?:string;byteLength:number}[];images?:unknown[];textures?:unknown[];extensionsRequired?:string[];meshes?:{primitives:{attributes:{POSITION?:number};mode?:number}[]}[];accessors?:{type:string;count:number}[]};
/** Embedded geometry only. No URL fetches delegated to GLTFLoader. */
export function inspectGlb(bytes:Uint8Array){
 const fail=()=>{throw new Error('preview-unit-data-error');};
 if(bytes.byteLength<20||bytes.byteLength>32*1024*1024)fail();
 const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
 if(view.getUint32(0,true)!==0x46546c67||view.getUint32(4,true)!==2||view.getUint32(8,true)!==bytes.byteLength||view.getUint32(16,true)!==0x4e4f534a)fail();
 const length=view.getUint32(12,true);if(length%4||length+20>bytes.byteLength)fail();
 const json=JSON.parse(new TextDecoder().decode(bytes.subarray(20,20+length))) as GlbJson;
 if(json.asset?.version!=='2.0'||json.images?.length||json.textures?.length||json.extensionsRequired?.length||!json.buffers?.length||json.buffers.some(b=>b.uri!==undefined)||!json.meshes?.length)fail();
 let vertices=0,triangles=0;
 for(const mesh of json.meshes!)for(const primitive of mesh.primitives){
  const accessor=json.accessors?.[primitive.attributes.POSITION??-1];
  if(!accessor||accessor.type!=='VEC3'||accessor.count<3||(primitive.mode??4)!==4)fail();
  vertices+=accessor!.count;triangles+=accessor!.count/3;
 }
 if(!vertices)fail();return {vertices,triangles,meshes:json.meshes!.length};
}

export async function loadUnitAssets(manifest:UnitAsset[],signal:AbortSignal){
 const loaded=new Map<string,Group>();const loader=new GLTFLoader();
 try{
  for(const asset of manifest){
   if(!['army','air','navy'].includes(asset.id)||asset.file!==`${asset.id}.glb`||!/^[a-f0-9]{64}$/.test(asset.sha256)||loaded.has(asset.id))throw new Error('preview-unit-data-error');
   const response=await fetch(`/preview/units3d/${asset.file}`,{signal,mode:'same-origin',redirect:'error'});
   if(!response.ok)throw new Error('preview-unit-data-error');const bytes=new Uint8Array(await response.arrayBuffer());
   if(await sha256(bytes)!==asset.sha256)throw new Error('preview-unit-data-error');
   inspectGlb(bytes);const gltf=await loader.parseAsync(new Uint8Array(bytes).buffer,'');
   if(signal.aborted){disposeUnits([gltf.scene]);throw new Error('preview-unit-aborted');}
   loaded.set(asset.id,gltf.scene);
  }
  if(loaded.size!==3)throw new Error('preview-unit-data-error');return loaded;
 }catch(error){disposeUnits(loaded.values());throw error;}
}

export function disposeUnits(models:Iterable<Group>){
 const geometries=new Set<BufferGeometry>(),materials=new Set<Material>();
 for(const model of models)model.traverse(object=>{if(object instanceof Mesh){geometries.add(object.geometry);for(const material of Array.isArray(object.material)?object.material:[object.material])materials.add(material);}});
 geometries.forEach(g=>g.dispose());materials.forEach(m=>m.dispose());
}

export type UnitLighting={ambient_intensity:number;sun_intensity:number;sun_position:[number,number,number];camera_far:number;camera_height:number};
/** Real meshes in the map's depth-tested scene. Decorations have no game IDs. */
export function unitScene(models:Map<string,Group>,samples:UnitSample[],width:number,height:number,lighting:UnitLighting,observe:(count:number,triangles:number)=>void){
 return async(scene:Scene,camera:OrthographicCamera)=>{
  const group=new Group();group.name='preview-unit-samples';let triangles=0;
  for(const sample of samples){
   const original=models.get(sample.asset);if(!original)throw new Error('preview-unit-data-error');
   const mesh=original.clone(true);mesh.name=`sample-${sample.asset}`;
   // +Y up / +Z forward asset -> map XY / +Z up. A display tilt exposes side faces.
   mesh.rotation.set(Math.PI/2-sample.tilt_degrees*Math.PI/180,0,-sample.heading_degrees*Math.PI/180,'ZXY');
   mesh.scale.setScalar(sample.scale);
   mesh.position.set(sample.longitude*width/360,sample.latitude*height/180,sample.height);
   mesh.traverse(object=>{if(object instanceof Mesh){object.frustumCulled=true;triangles+=object.geometry.index?object.geometry.index.count/3:object.geometry.attributes.position.count/3;}});
   // Tilted forward extremities stay above the map plane; preserve depth ordering.
   const bound=new Box3().setFromObject(mesh);if(bound.min.z<sample.height)mesh.position.z+=sample.height-bound.min.z;
   group.add(mesh);
  }
  const ambient=new AmbientLight(0xffffff,lighting.ambient_intensity),sun=new DirectionalLight(0xfff1cf,lighting.sun_intensity);
  sun.position.set(...lighting.sun_position);group.add(ambient,sun);scene.add(group);
  camera.position.z=lighting.camera_height;camera.far=lighting.camera_far;camera.updateProjectionMatrix();observe(samples.length,triangles);
  return ()=>{scene.remove(group);group.clear();};
 };
}
