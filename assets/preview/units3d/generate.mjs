// Self-authored deterministic preview-art GLB generator. Node built-ins only.
// Geometry/material values live in generate.json; none are simulation values.
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const leaf = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(leaf, '../../..');
const recipe = JSON.parse(fs.readFileSync(path.join(leaf, 'generate.json'), 'utf8'));
const output = process.argv[2] ? path.resolve(process.argv[2]) : leaf;
fs.mkdirSync(output, { recursive: true });
const sub = (a,b) => a.map((v,i)=>v-b[i]);
const cross = (a,b) => [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
const dot = (a,b) => a.reduce((s,v,i)=>s+v*b[i],0);
const average = vertices => vertices[0].map((_,i)=>vertices.reduce((s,v)=>s+v[i],0)/vertices.length);
const sha = bytes => crypto.createHash('sha256').update(bytes).digest('hex');

function layers(polygon, low, high, axis, upperScale=[1,1]) {
  const xyz = (p, t) => axis==='x' ? [t,p[1],p[0]] : axis==='y' ? [p[0],t,p[1]] : [p[0],p[1],t];
  const n=polygon.length;
  const vertices=[...polygon.map(p=>xyz(p,low)),...polygon.map(p=>xyz(p.map((v,i)=>v*upperScale[i]),high))];
  const faces=[Array.from({length:n},(_,i)=>i),Array.from({length:n},(_,i)=>i+n)];
  for(let i=0;i<n;i++) faces.push([i,(i+1)%n,(i+1)%n+n,i+n]);
  return {vertices,faces};
}

function geometry(part) {
  let raw;
  if(part.shape==='layers') raw=layers(part.polygon,part.low,part.high,part.axis,part.upper_scale);
  else if(part.shape==='box') {
    const [x,y,z]=part.size.map(v=>v/2);
    raw=layers([[-x,-z],[x,-z],[x,z],[-x,z]],-y,y,'y');
    raw.vertices=raw.vertices.map(v=>v.map((q,i)=>q+part.center[i]));
  } else if(part.shape==='cylinder') {
    const polygon=Array.from({length:part.segments},(_,i)=>[Math.cos(i*Math.PI*2/part.segments)*part.radius,Math.sin(i*Math.PI*2/part.segments)*part.radius]);
    const ratio=part.radius_end/part.radius;
    raw=layers(polygon,-part.length/2,part.length/2,part.axis,[ratio,ratio]);
    raw.vertices=raw.vertices.map(v=>v.map((q,i)=>q+part.center[i]));
  } else throw new Error(`unknown shape: ${part.shape}`);
  const centroid=average(raw.vertices), positions=[], normals=[], indices=[];
  for(const source of raw.faces) {
    let face=[...source];
    let normal=cross(sub(raw.vertices[face[1]],raw.vertices[face[0]]),sub(raw.vertices[face[2]],raw.vertices[face[0]]));
    if(dot(normal,sub(average(face.map(i=>raw.vertices[i])),centroid))<0) {face.reverse();normal=normal.map(v=>-v);}
    const magnitude=Math.hypot(...normal);
    if(magnitude<1e-10) throw new Error(`${part.name}: degenerate face`);
    normal=normal.map(v=>v/magnitude);
    for(let i=1;i<face.length-1;i++) {
      const triangle=[face[0],face[i],face[i+1]];
      let tn=cross(sub(raw.vertices[triangle[1]],raw.vertices[triangle[0]]),sub(raw.vertices[triangle[2]],raw.vertices[triangle[0]]));
      if(dot(tn,sub(average(triangle.map(id=>raw.vertices[id])),centroid))<0) {triangle.reverse();tn=tn.map(v=>-v);}
      const length=Math.hypot(...tn);
      if(length<1e-10) throw new Error(`${part.name}: degenerate triangle`);
      const base=positions.length/3;
      for(const id of triangle) {positions.push(...raw.vertices[id]);normals.push(...tn.map(v=>v/length));}
      indices.push(base,base+1,base+2);
    }
  }
  return {positions,normals,indices};
}

function glb(name, parts) {
  const binary=[], views=[], accessors=[], meshes=[], nodes=[];
  let offset=0;
  function accessor(values, kind, componentType, target) {
    const array=componentType===5126?new Float32Array(values):new Uint16Array(values);
    const bytes=Buffer.from(array.buffer);
    views.push({buffer:0,byteOffset:offset,byteLength:bytes.length,target});
    binary.push(bytes); offset+=bytes.length;
    const pad=(4-offset%4)%4; if(pad){binary.push(Buffer.alloc(pad));offset+=pad;}
    const width=kind==='VEC3'?3:1;
    const entry={bufferView:views.length-1,componentType,count:values.length/width,type:kind};
    if(target===34962) {entry.min=Array.from({length:width},(_,i)=>Math.min(...values.filter((_,j)=>j%width===i)));entry.max=Array.from({length:width},(_,i)=>Math.max(...values.filter((_,j)=>j%width===i)));}
    accessors.push(entry);return accessors.length-1;
  }
  for(const part of parts) {
    const g=geometry(part);
    const position=accessor(g.positions,'VEC3',5126,34962);
    const normal=accessor(g.normals,'VEC3',5126,34962);
    const index=accessor(g.indices,'SCALAR',5123,34963);
    meshes.push({name:part.name,primitives:[{attributes:{POSITION:position,NORMAL:normal},indices:index,material:part.material,mode:4}]});
    nodes.push({name:part.name,mesh:meshes.length-1});
  }
  const json={asset:{version:'2.0',generator:'OpenHOI4 self-authored preview-art generate.mjs v1'},scene:0,scenes:[{name:`preview-${name}`,nodes:nodes.map((_,i)=>i)}],nodes,meshes,
    materials:recipe.materials.map(m=>({name:m.name,pbrMetallicRoughness:{baseColorFactor:m.baseColorFactor,metallicFactor:m.metallicFactor,roughnessFactor:m.roughnessFactor},doubleSided:false,alphaMode:'OPAQUE'})),
    buffers:[{byteLength:offset}],bufferViews:views,accessors,
    extras:{assetId:`preview.units3d.${name}`,units:recipe.units,up:'+Y',forward:'+Z',pivot:[0,0,0],previewOnly:true,synthetic:true,actualDeployment:false,aiGenerated:false,recipeSchema:recipe.schema}};
  const jsonBytes=Buffer.from(JSON.stringify(json));
  const jsonPad=Buffer.alloc((4-jsonBytes.length%4)%4,0x20);
  const jsonChunk=Buffer.concat([jsonBytes,jsonPad]);
  const binChunk=Buffer.concat(binary);
  const header=Buffer.alloc(12);header.writeUInt32LE(0x46546c67);header.writeUInt32LE(2,4);header.writeUInt32LE(12+8+jsonChunk.length+8+binChunk.length,8);
  const jh=Buffer.alloc(8);jh.writeUInt32LE(jsonChunk.length);jh.writeUInt32LE(0x4e4f534a,4);
  const bh=Buffer.alloc(8);bh.writeUInt32LE(binChunk.length);bh.writeUInt32LE(0x004e4942,4);
  return Buffer.concat([header,jh,jsonChunk,bh,binChunk]);
}

const results=[];
for(const [name,parts] of Object.entries(recipe.models)) {
  const bytes=glb(name,parts), destination=path.join(output,`${name}.glb`);
  fs.writeFileSync(destination,bytes);
  results.push({name,path:destination,bytes:bytes.length,sha256:sha(bytes)});
  if(output===leaf) {
    const client=path.join(root,'client/public/preview/units3d'); fs.mkdirSync(client,{recursive:true});
    fs.copyFileSync(destination,path.join(client,`${name}.glb`));
  }
}
console.log(JSON.stringify({recipe_sha256:sha(fs.readFileSync(path.join(leaf,'generate.json'))),generator_sha256:sha(fs.readFileSync(fileURLToPath(import.meta.url))),models:results},null,2));
