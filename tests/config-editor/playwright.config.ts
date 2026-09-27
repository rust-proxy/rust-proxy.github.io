import { defineConfig } from '@playwright/test';
import { target } from './playwright.base';

// Standalone local build served at the root; used by `npm test` and `just browser`.
export default defineConfig(
  target({
    directory: 'config-editor/dist',
    prefix: '/',
    port: 4173,
    testMatch: ['editor.spec.ts', 'visual.spec.ts'],
  }),
);
