import { test as base, expect, type Locator, type Page } from '@playwright/test';

interface Fixtures {
  externalRequests: string[];
}

// Track off-origin requests and fail the test if the editor makes any.
export const test = base.extend<Fixtures>({
  externalRequests: [
    async ({ page, baseURL }, use) => {
      const external: string[] = [];
      const origin = new URL(baseURL ?? 'http://127.0.0.1').origin;
      await page.route('**/*', route => {
        const url = route.request().url();
        if (url.startsWith(origin) || url.startsWith('data:') || url.startsWith('blob:')) return route.continue();
        external.push(url);
        return route.abort();
      });
      await use(external);
      expect(external, 'the editor must not request external resources').toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };

export function field(page: Page, key: string): Locator {
  return page.getByTestId(`field-${key}`);
}

export function previewLines(page: Page): Locator {
  return page.locator('.preview-doc__line');
}

export async function previewText(page: Page): Promise<string> {
  return (await page.locator('.preview-doc__line code').allTextContents()).join('\n');
}

export function lineWith(page: Page, text: string): Locator {
  return previewLines(page).filter({ hasText: text }).first();
}

export function section(page: Page, label: string): Locator {
  return page.locator('summary', { hasText: label });
}

export async function click(page: Page, name: string): Promise<void> {
  await page.getByRole('button', { name, exact: true }).click();
}

export async function waitReady(page: Page): Promise<void> {
  await expect(page.locator('#editor')).toHaveAttribute('data-ready', 'true');
}
