import {defineConfig} from '@playwright/test';
import base from './playwright.m1.config';
export default defineConfig({...base,use:{...base.use,trace:'on'},reporter:[['line'],['json',{outputFile:process.env.OH_RESIZE_PHASE_EVIDENCE+'/report.json'}]],outputDir:process.env.OH_RESIZE_PHASE_EVIDENCE+'/results'});
