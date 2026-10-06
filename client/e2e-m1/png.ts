import { inflateSync } from 'node:zlib';
/** Minimal lossless RGB/RGBA PNG decoder for actual browser screenshots. */
export function png(buffer:Buffer){
 let pos=8,width=0,height=0,channels=0;const chunks:Buffer[]=[];
 while(pos<buffer.length){const size=buffer.readUInt32BE(pos),kind=buffer.toString('ascii',pos+4,pos+8),data=buffer.subarray(pos+8,pos+8+size);if(kind==='IHDR'){width=data.readUInt32BE(0);height=data.readUInt32BE(4);if(data[8]!==8||![2,6].includes(data[9]))throw new Error('unsupported screenshot PNG');channels=data[9]===6?4:3;}if(kind==='IDAT')chunks.push(data);pos+=size+12;}
 const raw=inflateSync(Buffer.concat(chunks)),stride=width*channels,out=Buffer.alloc(height*stride);
 const paeth=(a:number,b:number,c:number)=>{const p=a+b-c,pa=Math.abs(p-a),pb=Math.abs(p-b),pc=Math.abs(p-c);return pa<=pb&&pa<=pc?a:pb<=pc?b:c;};
 for(let y=0;y<height;y++){const filter=raw[y*(stride+1)];for(let x=0;x<stride;x++){const left=x>=channels?out[y*stride+x-channels]:0,up=y?out[(y-1)*stride+x]:0,corner=y&&x>=channels?out[(y-1)*stride+x-channels]:0;const predictor=[0,left,up,Math.floor((left+up)/2),paeth(left,up,corner)][filter];if(predictor===undefined)throw new Error('PNG filter');out[y*stride+x]=(raw[y*(stride+1)+1+x]+predictor)&255;}}
 return {width,height,pixel:(x:number,y:number)=>Array.from(out.subarray(Math.floor(y)*stride+Math.floor(x)*channels,Math.floor(y)*stride+Math.floor(x)*channels+3))};
}
