import { expect, field, test, waitReady } from './fixtures';
import type { Page } from '@playwright/test';

const themes = ['latte', 'frappe', 'macchiato', 'mocha'] as const;

// Wait for webfonts and drop the caret so screenshots are deterministic.
async function settle(page: Page): Promise<void> {
  await page.evaluate(() => document.fonts.ready);
  await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
}

// Visual baselines are generated and compared on Linux so fonts stay stable.
for (const theme of themes) {
  test(`renders the ${theme} theme on desktop and mobile`, async ({ page }) => {
    test.skip(process.platform !== 'linux', 'visual baselines are generated on Linux');
    await page.setViewportSize({ width: 1440, height: 1000 });
    await page.goto('?schema=tuic-client');
    await waitReady(page);
    await field(page, 'users.0.uuid').fill('00000000-0000-4000-8000-000000000000');
    await field(page, 'users.0.password').fill('snapshot-password');
    await field(page, 'host').fill('tuic.example.com');
    await page.getByRole('combobox', { name: '主题' }).selectOption(theme);
    await settle(page);
    await expect(page).toHaveScreenshot(`editor-${theme}.png`);
    await page.setViewportSize({ width: 390, height: 844 });
    await settle(page);
    await expect(page).toHaveScreenshot(`editor-${theme}-mobile.png`);
  });
}
