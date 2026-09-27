import { defineConfig } from '@playwright/test';
import { target } from './playwright.base';

// Assembled Pages build at the deployment prefix; used by CI and `just browser-site`.
export default defineConfig(
  target({
    directory: 'site',
    prefix: '/config-editor/',
    port: 4174,
    testMatch: ['editor.spec.ts', 'visual.spec.ts'],
  }),
);
