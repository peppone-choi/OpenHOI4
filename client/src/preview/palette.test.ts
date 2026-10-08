import { it,expect } from 'vitest';
import { previewPalette } from './palette';
import { palettes,updateColors } from '../map/model';
import { previewWorld,type PreviewMetadata,type PreviewProvince } from './model';
import { readFileSync } from 'node:fs';
import { deepStrictEqual } from 'node:assert/strict';
it('preview indexed palette is byte-identical to existing display palettes and update ranges',()=>{
 const meta=JSON.parse(readFileSync('public/preview/world/metadata.json','utf8')) as PreviewMetadata;
 const provinces=JSON.parse(readFileSync('public/preview/world/provinces.json','utf8')) as PreviewProvince[];
 const world=previewWorld(meta,provinces,'geography'),fast=previewPalette.palettes(world,'map-mode-terrain');
 // Node compares typed-array bytes directly, preserving full palette equality
 // without Vitest walking every numeric property in the large lookup buffers.
 deepStrictEqual(fast,palettes(world,'map-mode-terrain'));
 const next=previewWorld(meta,provinces,'provinces'),a=fast.colors.slice(),b=a.slice();
 expect(previewPalette.updateColors(a,next,'map-mode-terrain')).toEqual(updateColors(b,next,'map-mode-terrain'));deepStrictEqual(a,b);
});
