import { click, expect, field, previewLines, previewText, test, waitReady } from './fixtures';
import { checkLayout } from './layout';

async function output(page: Parameters<typeof previewText>[0]): Promise<Record<string, any>> {
  return JSON.parse(await previewText(page)) as Record<string, any>;
}

const scheme = (page: Parameters<typeof previewText>[0]) => page.getByRole('combobox', { name: '配置方案' });

test('alternate XML description drives forms, resets, validation and three outputs', async ({ page }) => {
  await page.goto('?schema=example');
  await waitReady(page);
  await expect(page).toHaveTitle('Notebook 任务清单编辑器');
  await expect(scheme(page)).toHaveValue('example');
  await expect(field(page, 'project')).toHaveCount(1);
  expect((await previewLines(page).allTextContents()).some(line => line.includes('"title"'))).toBe(true);
  await expect(previewLines(page).filter({ hasText: '"secret"' }).first()).toHaveAttribute('aria-label', /敏感/);

  await field(page, 'encoding').selectOption('toml');
  expect(await previewText(page)).toContain('[[items]]');
  expect(await page.locator('.preview-doc__line code .token.class-name').count()).toBeGreaterThan(0);
  await field(page, 'encoding').selectOption('yaml');
  expect(await previewText(page)).toContain('title: "example"');
  expect(await page.locator('.preview-doc__line code .token.atrule').count()).toBeGreaterThan(0);
  await field(page, 'encoding').selectOption('json');

  expect((await field(page, 'seed').inputValue()).length).toBe(16);
  const seed = await field(page, 'seed').inputValue();
  await field(page, 'confirm').check();
  await click(page, '生成随机值');
  expect(await field(page, 'seed').inputValue()).not.toBe(seed);
  expect(await field(page, 'confirm').isChecked()).toBe(false);
  expect((await output(page)).items[0].enabled).toBe(true);

  await field(page, 'entries.0.enabled').uncheck();
  expect((await output(page)).items[0].enabled).toBe(false);
  await field(page, 'entries.0.id').fill('kept');
  const token = await field(page, 'entries.0.token').inputValue();
  await click(page, '更新标记');
  expect(await field(page, 'entries.0.id').inputValue()).toBe('kept');
  expect(await field(page, 'entries.0.enabled').isChecked()).toBe(false);
  expect(await field(page, 'entries.0.token').inputValue()).not.toBe(token);
  expect((await output(page)).items[0].secret).toBe('••••••••');

  await page.getByRole('checkbox', { name: '显示密码' }).check();
  expect((await output(page)).items[0].secret.length).toBe(24);
  await click(page, '添加项目');
  await field(page, 'entries.1.id').fill('second');
  const retainedRow = await field(page, 'entries.1.id').elementHandle();
  await field(page, 'chosen').selectOption('1');
  await click(page, '单项清单');
  expect((await output(page)).item).toBe('second');
  await click(page, '移除项目 1');
  expect((await output(page)).item).toBe('second');
  expect(await retainedRow!.evaluate(node => node.isConnected && node.id === 'field-entries.0.id')).toBe(true);
  await field(page, 'entries.0.id').focus();
  await field(page, 'entries.0.id').pressSequentially('-edited');
  expect(await retainedRow!.evaluate(node => document.activeElement === node)).toBe(true);
  await field(page, 'entries.0.id').fill('second');

  await click(page, '单个项目');
  await expect(page.getByRole('button', { name: '完整清单', exact: true })).toBeHidden();
  await field(page, 'confirm').check();
  await click(page, '审计信息');
  expect((await output(page)).confirmed).toBe(true);
  await field(page, 'project').fill('changed');
  expect(await field(page, 'confirm').isChecked()).toBe(false);
  expect((await output(page)).item).toBe('second');

  await click(page, '全部清单');
  await click(page, '完整清单');
  await field(page, 'encoding').selectOption('toml');
  await expect(page.getByTestId('export-command')).toHaveText('notebook --input snapshot.toml');
  await field(page, 'first').fill('50');
  await expect(page.getByRole('button', { name: '下载配置', exact: true })).toBeDisabled();
  const validationTarget = await page.locator('[aria-invalid="true"]').first().getAttribute('id');
  await page.getByTestId('error-link').first().click();
  expect(await page.evaluate(() => document.activeElement?.id)).toBe(validationTarget);
  await field(page, 'last').fill('60');
  await expect(page.getByRole('button', { name: '下载配置', exact: true })).toBeEnabled();

  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await checkLayout(page, { field: 'first', secret: 'entries.0.token' });
});

test('failed Crypto API calls preserve state and allow manual editing', async ({ browser, baseURL }) => {
  const context = await browser.newContext({ baseURL });
  await context.addInitScript(() => {
    let calls = 0;
    Object.defineProperty(Crypto.prototype, 'getRandomValues', {
      value(bytes: Uint8Array) {
        if (++calls > 1) throw new DOMException('Random unavailable');
        return bytes.fill(0xab);
      },
    });
  });
  const page = await context.newPage();
  try {
    await page.goto('/');
    await waitReady(page);
    await expect(field(page, 'seed')).toHaveValue('');
    await expect(page.getByTestId('status-bar')).toContainText('无法安全生成随机值');
    await click(page, '添加项目');
    await expect(page.locator('[data-row]')).toHaveCount(1);
    await field(page, 'project').fill('manual');
    await expect(field(page, 'project')).toHaveValue('manual');
  } finally {
    await context.close();
  }
});
