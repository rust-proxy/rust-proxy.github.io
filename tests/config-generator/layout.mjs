import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

export async function checkLayout(page, { field, secret, name }) {
  const input = page.locator(`[id="cg-${field}"]`);
  const search = page.getByRole('searchbox', { name: '查找配置项' });
  const edit = page.getByRole('button', { name: '编辑配置', exact: true });
  const preview = page.getByRole('button', { name: /^预览与导出/ });
  await page.setViewportSize({ width: 1440, height: 1000 });
  await search.fill(await page.locator(`[id="cg-${secret}"]`).inputValue());
  assert.equal(await page.locator('.cg-search-results li').count(), 0, 'Search never indexes input values');
  await search.fill('not-a-field-xyz');
  await search.press('Escape');
  assert.equal(await search.inputValue(), '');
  await search.fill(field);
  await page.locator('.cg-search-results button').filter({ hasText: field }).first().click();
  assert.equal(await input.evaluate(node => document.activeElement === node), true, 'Search focuses the matching field');
  const previous = await input.inputValue();
  await input.fill('not-a-number');
  await page.setViewportSize({ width: 390, height: 844 });
  await preview.click();
  assert.equal(await input.isVisible(), false);
  assert.equal(await page.locator('#cg-preview').isVisible(), true);
  await page.locator('.cg-errors button').filter({ hasText: await page.locator(`[id="cg-${field}-error"]`).textContent() }).first().click();
  assert.equal(await input.isVisible(), true);
  assert.equal(await input.evaluate(node => document.activeElement === node), true, 'Mobile error navigation switches panes before focusing');
  await input.fill(previous);
  const editorScroll = await page.evaluate(() => window.scrollY);
  await preview.click();
  await edit.click();
  assert.ok(Math.abs(await page.evaluate(() => window.scrollY) - editorScroll) < 2, 'Returning to editing restores its scroll position');
  assert.equal(await input.inputValue(), previous, 'Pane switching preserves edits');
  mkdirSync(resolve('.cache'), { recursive: true });
  for (const width of [1440, 1280, 1024, 768, 390, 320]) {
    await page.setViewportSize({ width, height: 900 });
    await page.evaluate(() => window.scrollTo(0, 0));
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${width}px editor does not overflow`);
    await page.screenshot({ path: resolve(`.cache/${name}-layout-${width}.png`) });
    if (width < 960) {
      await preview.click();
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `${width}px preview does not overflow`);
      await page.screenshot({ path: resolve(`.cache/${name}-preview-${width}.png`) });
      await edit.click();
    }
  }
  await page.setViewportSize({ width: 1280, height: 600 });
  const download = page.getByRole('button', { name: '下载配置', exact: true });
  await download.scrollIntoViewIfNeeded();
  const bounds = await download.boundingBox();
  assert.ok(bounds.y >= 0 && bounds.y + bounds.height <= 600, 'Export remains reachable in short windows');
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.getByRole('navigation', { name: '配置分区' }).getByRole('button').last().click();
  await page.waitForFunction(() => {
    const buttons = [...document.querySelectorAll('.cg-section-nav > button')];
    return buttons.at(-1)?.getAttribute('aria-current') === 'location';
  });
  console.log(`PASS: ${name} search, mobile error focus, view switching and responsive layouts`);
}
