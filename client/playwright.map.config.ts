import { defineConfig } from '@playwright/test';
import m1 from './playwright.m1.config';
export default defineConfig({...m1,testMatch:['map.spec.ts','map-redirect.spec.ts']});
