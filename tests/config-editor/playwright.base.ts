import { devices, type PlaywrightTestConfig } from '@playwright/test';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const configDir = dirname(fileURLToPath(import.meta.url));
const root = resolve(configDir, '..', '..');

interface Target {
  directory: string;
  prefix: string;
  port: number;
  testMatch: string[];
}

/// Shared settings for the standalone, assembled and generic builds.
export function target({ directory, prefix, port, testMatch }: Target): PlaywrightTestConfig {
  const baseURL = `http://127.0.0.1:${port}${prefix}`;
  return {
    testDir: './e2e',
    testMatch,
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 1 : 0,
    reporter: [['list'], ['html', { open: 'never' }]],
    outputDir: './test-results',
    snapshotPathTemplate: '{testDir}/__screenshots__/{platform}/{projectName}/{arg}{ext}',
    expect: {
      toHaveScreenshot: { maxDiffPixelRatio: 0.02, animations: 'disabled' },
    },
    use: {
      baseURL,
      trace: 'on-first-retry',
      screenshot: 'only-on-failure',
      video: 'retain-on-failure',
    },
    webServer: {
      command: `node ${resolve(configDir, 'serve.mjs')} --dir ${resolve(root, directory)} --port ${port}`,
      url: baseURL,
      reuseExistingServer: !process.env.CI,
    },
    projects: [
      {
        name: 'chromium',
        use: { ...devices['Desktop Chrome'], permissions: ['clipboard-read', 'clipboard-write'] },
      },
      { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    ],
  };
}
