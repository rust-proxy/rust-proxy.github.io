import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { playwright } from '@vitest/browser-playwright';

export default defineConfig({
  plugins: [svelte()],
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: 'unit',
          environment: 'jsdom',
          include: ['ui/**/*.spec.ts'],
          exclude: ['ui/**/*.browser.spec.ts'],
          clearMocks: true,
          unstubGlobals: true,
        },
      },
      {
        extends: true,
        test: {
          name: 'browser',
          include: ['ui/**/*.browser.spec.ts'],
          setupFiles: ['./ui/testing/setup.browser.ts'],
          browser: {
            enabled: true,
            headless: true,
            provider: playwright(),
            instances: [{ browser: 'chromium' }, { browser: 'firefox' }],
          },
        },
      },
    ],
  },
});
