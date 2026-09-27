import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { mkdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { checkLayout } from './layout.mjs';

const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE_PATH ?? 'playwright');
const base = process.env.PREVIEW_URL ?? 'http://127.0.0.1:8765/config-editor/';
const channel = process.env.BROWSER_CHANNEL ?? (process.env.CI ? 'chromium' : 'msedge');
const browser = await chromium.launch({ channel, headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, permissions: ['clipboard-read', 'clipboard-write'] });
const page = await context.newPage();
const errors = [];
const requests = [];
const localFailures = [];
page.on('pageerror', error => errors.push(error.message));
page.on('request', request => requests.push(request.url()));
page.on('response', response => {
  if (response.url().startsWith(new URL(base).origin) && response.status() >= 400) localFailures.push(response.status());
});
await page.route('**/*', route => route.request().url().startsWith(new URL(base).origin) ? route.continue() : route.abort());
const field = key => page.locator(`[id="field-${key}"]`);
const chrome = key => page.locator(`#${key}`);
const lines = () => page.locator('.preview-doc__line');
const previewText = async () => (await page.locator('.preview-doc__line code').allTextContents()).join('\n');
const click = name => page.getByRole('button', { name, exact: true }).click();
const section = label => page.locator('summary', { hasText: label });
async function output() {
  await field('format').selectOption('json');
  return JSON.parse(await previewText());
}
async function lineWith(text) {
  return lines().filter({ hasText: text }).first();
}

try {
  await page.goto(`${base}?schema=tuic-client&retained=yes#query-state`);
  await page.waitForSelector('#editor[data-ready="true"]');
  assert.equal(await chrome('editor-schema').inputValue(), 'tuic-client');
  assert.equal(await field('host').count(), 1, 'selection region shows the editor form');

  await field('host').fill('tuic.example.com');
  assert.ok((await lines().allTextContents()).some(line => line.includes('skip_cert_verify')), 'preview region shows the generated config');
  const skipLine = await lineWith('skip_cert_verify');
  assert.match(await skipLine.getAttribute('aria-label'), /证书链和主机名验证/, 'preview lines carry field explanations');
  await skipLine.hover();
  const tooltip = page.locator('[role="tooltip"]');
  await tooltip.waitFor({ state: 'visible' });
  assert.match(await tooltip.textContent(), /证书链和主机名验证/);
  await skipLine.focus();
  await page.keyboard.press('Escape');
  assert.equal(await tooltip.count(), 0, 'Escape dismisses the keyboard preview explanation');
  await field('format').selectOption('toml');
  assert.ok((await lines().allTextContents()).some(line => line.includes('[tls]')), 'TOML tables are highlighted');
  await field('format').selectOption('yaml');
  assert.ok((await lines().allTextContents()).some(line => line.includes('skip_cert_verify: false')));
  assert.ok(await page.locator('.preview-doc__line code .token.atrule').count() > 0, 'YAML keys are highlighted');
  await field('format').selectOption('json');

  let client = await output();
  assert.equal(client.server, 'tuic.example.com:8443');
  assert.equal(client.tls.skip_cert_verify, false);
  await chrome('editor-reveal').check();
  client = await output();
  assert.equal(typeof client.password, 'string');
  await click('复制配置');
  const copied = JSON.parse(await page.evaluate(() => navigator.clipboard.readText()));
  assert.equal(copied.password, client.password);
  const pendingDownload = page.waitForEvent('download');
  await click('下载配置');
  const download = await pendingDownload;
  assert.equal(download.suggestedFilename(), 'client.json');
  assert.equal(JSON.parse(readFileSync(await download.path(), 'utf8')).password, copied.password);
  await chrome('editor-reveal').uncheck();
  assert.equal((await output()).password, '••••••••', 'preview hides secrets until revealed');

  await click('＋ 添加用户');
  assert.equal(await field('users.1.uuid').evaluate(node => document.activeElement === node), true, 'Adding a row focuses its first field');
  await field('activeUser').selectOption('1');
  client = await output();
  assert.equal(client.uuid, await field('users.1.uuid').inputValue());
  await page.getByRole('button', { name: '移除用户 1', exact: true }).click();
  assert.equal(await field('users.0.uuid').evaluate(node => document.activeElement === node), true, 'Removing a row moves focus to the remaining row');
  assert.equal((await output()).uuid, client.uuid);

  await field('port').fill('443');
  await field('host').fill('[2001:db8::1]');
  await field('sni').fill('tuic.example.com');
  client = await output();
  assert.equal(client.server, '[2001:db8::1]:443');
  assert.equal(client.ip, '2001:db8::1');
  await section('高级选项').click();
  await field('localAuth').check();
  await field('localUsername').fill('tester');
  await field('localPassword').fill('a " \\ # 中文');
  await click('＋ 添加转发');
  await field('forwards.0.listen').fill('127.0.0.1:8080');
  await field('forwards.0.remote').fill('example.com:80');
  await field('forwards.0.protocol').selectOption('udp');
  await field('forwards.0.timeout').fill('30');
  client = await output();
  assert.equal(client.local.udp_forward[0].timeout, '30s');
  await field('reconnect').uncheck();
  assert.equal((await output()).reconnect_initial_backoff, undefined);
  console.log('PASS: merged client preview, descriptions, export and forwarding');

  await chrome('editor-reveal').check();
  await chrome('editor-schema').selectOption('tuic-server');
  assert.equal(await chrome('editor-reveal').isChecked(), false, 'Schema changes reset the secret control as well as the preview');
  assert.equal(new URL(page.url()).searchParams.get('schema'), 'tuic-server');
  assert.equal(new URL(page.url()).searchParams.get('retained'), 'yes');
  assert.equal(new URL(page.url()).hash, '#query-state');
  assert.equal(await field('mode').count(), 0, 'Application schemas must not expose the old paired generation mode');
  assert.equal(await field('hostname').inputValue(), '', 'Switching schemas must create an independent session');
  await field('hostname').fill('tuic.example.com');
  let server = await output();
  assert.equal(server.server, '[::]:8443');
  assert.equal(Object.keys(server.users).length, 1);
  assert.equal(server.tls.certificate, '/etc/tuic/fullchain.pem');
  const certificateLine = await lineWith('"certificate"');
  assert.match(await certificateLine.getAttribute('aria-label'), /证书链文件路径/);
  await click('＋ 添加用户');
  server = await output();
  assert.equal(Object.keys(server.users).length, 2);

  await field('tlsMode').selectOption('self');
  server = await output();
  assert.equal(server.tls.self_sign, true);
  await field('tlsMode').selectOption('acme');
  await field('email').fill('admin@example.com');
  server = await output();
  assert.equal(server.tls.auto_ssl, true);
  assert.equal(server.tls.self_sign, undefined);
  const acmeLine = await lineWith('"auto_ssl"');
  assert.match(await acmeLine.getAttribute('aria-label'), /ACME/);
  await field('tlsMode').selectOption('certificate');
  server = await output();
  assert.equal(server.data_dir, undefined);

  await page.getByRole('navigation', { name: '配置分区' }).getByRole('button', { name: 'QUIC 后端' }).click();
  assert.equal(await page.locator('summary:focus').count(), 1, 'Section navigation opens and focuses the collapsed section');
  await field('backendMode').selectOption('quiche');
  server = await output();
  assert.equal(server.backend.mode, 'quiche');
  assert.equal(server.backend.quiche.max_concurrent_bi_streams, 100);
  await section('QUIC 后端').click();
  await section('路由与出站').click();
  await field('dnsEnabled').check();
  await field('dnsMode').selectOption('custom');
  await field('dnsServers.0.server').fill('tls://1.1.1.1#cloudflare-dns.com');
  server = await output();
  assert.equal(server.dns.servers[0], 'tls://1.1.1.1#cloudflare-dns.com');
  assert.equal(server.outbound.default.type, 'direct');
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: resolve('.cache/config-editor-desktop.png'), fullPage: true });
  console.log('PASS: merged server preview, TLS, backend and routing');

  const injection = '<img src=x onerror=alert(1)> " \\ 中文';
  await field('dnsServers.0.server').fill(injection);
  await field('format').selectOption('yaml');
  assert.ok((await previewText()).includes(injection.slice(0, 27)));
  await field('format').selectOption('toml');
  assert.equal(await page.locator('#editor img').count(), 0);
  await page.setViewportSize({ width: 390, height: 844 });
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth), true);
  await page.screenshot({ path: resolve('.cache/config-editor-mobile.png'), fullPage: true });
  await page.locator('#editor-theme').selectOption('mocha');
  assert.equal(await page.locator('.shell').getAttribute('data-theme'), 'mocha');
  await checkLayout(page, { field: 'serverAuthTimeout', secret: 'users.0.password', name: 'tuic' });
  assert.equal(await page.evaluate(() => localStorage.length === 0 || !Object.keys(localStorage).some(key => /editor|password|uuid/.test(key))), true);
  assert.deepEqual(requests.filter(url => /google-analytics|googletagmanager|gtag/.test(url)), []);
  assert.deepEqual(requests.filter(url => !url.startsWith(new URL(base).origin)), []);
  assert.deepEqual(localFailures, []);
  assert.deepEqual(errors, []);
  await page.reload();
  await page.waitForSelector('#editor[data-ready="true"]');
  assert.equal(await chrome('editor-schema').inputValue(), 'tuic-server');
  assert.equal(await field('hostname').inputValue(), '');
  console.log('PASS: URL state, injection safety, mobile layout, themes and no input persistence');
} finally {
  await context.close();
  await browser.close();
}
