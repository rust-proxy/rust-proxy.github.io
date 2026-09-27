import { expect, type Page } from '@playwright/test';
import { field } from './fixtures';

// Search, mobile pane switching, scroll memory and responsive overflow checks.
export async function checkLayout(page: Page, { field: fieldPath, secret }: { field: string; secret: string }): Promise<void> {
  const input = field(page, fieldPath);
  const search = page.getByRole('searchbox', { name: '查找配置项' });
  const edit = page.getByRole('button', { name: '编辑配置', exact: true });
  const preview = page.getByRole('button', { name: /^预览与导出/ });

  await page.setViewportSize({ width: 1440, height: 1000 });
  await search.fill(await field(page, secret).inputValue());
  await expect(page.getByTestId('search-results').locator('li')).toHaveCount(0, { timeout: 5000 });
  await search.fill('not-a-field-xyz');
  await search.press('Escape');
  await expect(search).toHaveValue('');
  await search.fill(fieldPath);
  await page.getByTestId('search-result').filter({ hasText: fieldPath }).first().click();
  await expect(input).toBeFocused();

  const previous = await input.inputValue();
  await input.fill('not-a-number');
  await page.setViewportSize({ width: 390, height: 844 });
  await preview.click();
  await expect(input).toBeHidden();
  await expect(page.locator('#editor-preview')).toBeVisible();
  const message = await page.locator(`[id="field-${fieldPath}-error"]`).textContent();
  await page.getByTestId('error-link').filter({ hasText: message ?? '' }).first().click();
  await expect(input).toBeVisible();
  await expect(input).toBeFocused();

  await input.fill(previous);
  const editorScroll = await page.evaluate(() => window.scrollY);
  await preview.click();
  await edit.click();
  expect(Math.abs((await page.evaluate(() => window.scrollY)) - editorScroll)).toBeLessThan(2);
  await expect(input).toHaveValue(previous);

  for (const width of [1440, 1280, 1024, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await page.evaluate(() => window.scrollTo(0, 0));
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `${width}px editor overflows`).toBe(true);
    if (width < 960) {
      await preview.click();
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), `${width}px preview overflows`).toBe(true);
      await edit.click();
    }
  }

  await page.setViewportSize({ width: 1280, height: 600 });
  const download = page.getByRole('button', { name: '下载配置', exact: true });
  await download.scrollIntoViewIfNeeded();
  const bounds = await download.boundingBox();
  expect(bounds).not.toBeNull();
  expect(bounds!.y >= 0 && bounds!.y + bounds!.height <= 600, 'export remains reachable in short windows').toBe(true);

  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.getByRole('navigation', { name: '配置分区' }).getByRole('button').last().click();
  await expect.poll(() => page.getByTestId('section-nav-item').last().getAttribute('aria-current')).toBe('location');
}
