import { readFileSync } from 'node:fs';
import { click, expect, field, lineWith, previewLines, previewText, section, test, waitReady } from './fixtures';
import { checkLayout } from './layout';

async function output(page: Parameters<typeof previewText>[0], format = 'json'): Promise<Record<string, any>> {
  await field(page, 'format').selectOption(format);
  return JSON.parse(await previewText(page)) as Record<string, any>;
}

const scheme = (page: Parameters<typeof previewText>[0]) => page.getByRole('combobox', { name: '配置方案' });
const reveal = (page: Parameters<typeof previewText>[0]) => page.getByRole('checkbox', { name: '显示密码' });

test('client schema previews, exports, reveals secrets and edits users and forwarding', async ({ page, browserName }) => {
  await page.goto('?schema=tuic-client');
  await waitReady(page);
  await expect(scheme(page)).toHaveValue('tuic-client');
  await expect(field(page, 'host')).toHaveCount(1, { timeout: 10000 });

  await field(page, 'host').fill('tuic.example.com');
  await expect(previewLines(page).filter({ hasText: 'skip_cert_verify' }).first()).toBeVisible();
  const skipLine = lineWith(page, 'skip_cert_verify');
  await expect(skipLine).toHaveAttribute('aria-label', /证书链和主机名验证/);
  await skipLine.hover();
  const tooltip = page.locator('[role="tooltip"]');
  await expect(tooltip).toBeVisible();
  await expect(tooltip).toContainText('证书链和主机名验证');
  await skipLine.focus();
  await page.keyboard.press('Escape');
  await expect(tooltip).toHaveCount(0);

  await field(page, 'format').selectOption('toml');
  expect(await previewText(page)).toContain('[tls]');
  await field(page, 'format').selectOption('yaml');
  expect(await previewText(page)).toContain('skip_cert_verify: false');
  expect(await page.locator('.preview-doc__line code .token.atrule').count()).toBeGreaterThan(0);
  await field(page, 'format').selectOption('json');

  let client = await output(page);
  expect(client.server).toBe('tuic.example.com:8443');
  expect(client.tls.skip_cert_verify).toBe(false);
  await reveal(page).check();
  client = await output(page);
  expect(typeof client.password).toBe('string');

  await click(page, '复制配置');
  if (browserName === 'chromium') {
    const copied = JSON.parse(await page.evaluate(() => navigator.clipboard.readText()));
    expect(copied.password).toBe(client.password);
  }

  const pendingDownload = page.waitForEvent('download');
  await click(page, '下载配置');
  const download = await pendingDownload;
  expect(download.suggestedFilename()).toBe('client.json');
  const downloadPath = await download.path();
  expect(downloadPath).not.toBeNull();
  expect(JSON.parse(readFileSync(downloadPath!, 'utf8')).password).toBe(client.password);

  await reveal(page).uncheck();
  expect((await output(page)).password).toBe('••••••••');

  await click(page, '＋ 添加用户');
  await expect(field(page, 'users.1.uuid')).toBeFocused();
  await field(page, 'activeUser').selectOption('1');
  client = await output(page);
  expect(client.uuid).toBe(await field(page, 'users.1.uuid').inputValue());
  await page.getByRole('button', { name: '移除用户 1', exact: true }).click();
  await expect(field(page, 'users.0.uuid')).toBeFocused();
  expect((await output(page)).uuid).toBe(client.uuid);

  await field(page, 'port').fill('443');
  await field(page, 'host').fill('[2001:db8::1]');
  await field(page, 'sni').fill('tuic.example.com');
  client = await output(page);
  expect(client.server).toBe('[2001:db8::1]:443');
  expect(client.ip).toBe('2001:db8::1');

  await section(page, '高级选项').click();
  await field(page, 'localAuth').check();
  await field(page, 'localUsername').fill('tester');
  await field(page, 'localPassword').fill('a " \\ # 中文');
  await click(page, '＋ 添加转发');
  await field(page, 'forwards.0.listen').fill('127.0.0.1:8080');
  await field(page, 'forwards.0.remote').fill('example.com:80');
  await field(page, 'forwards.0.protocol').selectOption('udp');
  await field(page, 'forwards.0.timeout').fill('30');
  client = await output(page);
  expect(client.local.udp_forward[0].timeout).toBe('30s');
  await field(page, 'reconnect').uncheck();
  expect((await output(page)).reconnect_initial_backoff).toBeUndefined();
});

test('server schema previews TLS modes, backend and routing safely', async ({ page }) => {
  await page.goto('?schema=tuic-server');
  await waitReady(page);
  await field(page, 'hostname').fill('tuic.example.com');
  let server = await output(page);
  expect(server.server).toBe('[::]:8443');
  expect(Object.keys(server.users).length).toBe(1);
  expect(server.tls.certificate).toBe('/etc/tuic/fullchain.pem');
  await expect(lineWith(page, '"certificate"')).toHaveAttribute('aria-label', /证书链文件路径/);
  await click(page, '＋ 添加用户');
  server = await output(page);
  expect(Object.keys(server.users).length).toBe(2);

  await field(page, 'tlsMode').selectOption('self');
  server = await output(page);
  expect(server.tls.self_sign).toBe(true);
  await field(page, 'tlsMode').selectOption('acme');
  await field(page, 'email').fill('admin@example.com');
  server = await output(page);
  expect(server.tls.auto_ssl).toBe(true);
  expect(server.tls.self_sign).toBeUndefined();
  await expect(lineWith(page, '"auto_ssl"')).toHaveAttribute('aria-label', /ACME/);
  await field(page, 'tlsMode').selectOption('certificate');
  server = await output(page);
  expect(server.data_dir).toBeUndefined();

  await page.getByRole('navigation', { name: '配置分区' }).getByRole('button', { name: 'QUIC 后端' }).click();
  await expect(page.locator('summary:focus')).toHaveCount(1);
  await field(page, 'backendMode').selectOption('quiche');
  server = await output(page);
  expect(server.backend.mode).toBe('quiche');
  expect(server.backend.quiche.max_concurrent_bi_streams).toBe(100);
  await section(page, 'QUIC 后端').click();
  await section(page, '路由与出站').click();
  await field(page, 'dnsEnabled').check();
  await field(page, 'dnsMode').selectOption('custom');
  await field(page, 'dnsServers.0.server').fill('tls://1.1.1.1#cloudflare-dns.com');
  server = await output(page);
  expect(server.dns.servers[0]).toBe('tls://1.1.1.1#cloudflare-dns.com');
  expect(server.outbound.default.type).toBe('direct');

  const injection = '<img src=x onerror=alert(1)> " \\ 中文';
  await field(page, 'dnsServers.0.server').fill(injection);
  await field(page, 'format').selectOption('yaml');
  expect(await previewText(page)).toContain(injection.slice(0, 27));
  await field(page, 'format').selectOption('toml');
  await expect(page.locator('#editor img')).toHaveCount(0);

  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await page.getByRole('combobox', { name: '主题' }).selectOption('mocha');
  await expect(page.locator('[data-theme]')).toHaveAttribute('data-theme', 'mocha');
  await checkLayout(page, { field: 'serverAuthTimeout', secret: 'users.0.password' });
});

test('URL state survives schema switching without persisting input', async ({ page }) => {
  await page.goto('?schema=tuic-client&retained=yes#query-state');
  await waitReady(page);
  await field(page, 'host').fill('example.com');
  await reveal(page).check();
  await scheme(page).selectOption('tuic-server');
  await expect(reveal(page)).not.toBeChecked();
  expect(new URL(page.url()).searchParams.get('schema')).toBe('tuic-server');
  expect(new URL(page.url()).searchParams.get('retained')).toBe('yes');
  expect(new URL(page.url()).hash).toBe('#query-state');
  await expect(field(page, 'mode')).toHaveCount(0);
  await expect(field(page, 'hostname')).toHaveValue('');

  await page.reload();
  await waitReady(page);
  await expect(scheme(page)).toHaveValue('tuic-server');
  await expect(field(page, 'hostname')).toHaveValue('');
  expect(await page.evaluate(() => localStorage.length === 0 || !Object.keys(localStorage).some(key => /editor|password|uuid/.test(key)))).toBe(true);
});
