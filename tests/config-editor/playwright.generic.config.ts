import { defineConfig } from '@playwright/test';
import { target } from './playwright.base';

// Reuse check built with `CONFIG_SCHEMA=schema/example.xml` into .cache/generic-site.
export default defineConfig(
  target({
    directory: '.cache/generic-site',
    prefix: '/',
    port: 4175,
    testMatch: ['generic.spec.ts'],
  }),
);
