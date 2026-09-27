// Dependency-free static server used by Playwright's `webServer` and local previews.
import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, normalize, resolve, sep } from 'node:path';

const types = new Map([
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.mjs', 'text/javascript; charset=utf-8'],
  ['.css', 'text/css; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.wasm', 'application/wasm'],
  ['.svg', 'image/svg+xml'],
  ['.png', 'image/png'],
  ['.jpg', 'image/jpeg'],
  ['.jpeg', 'image/jpeg'],
  ['.gif', 'image/gif'],
  ['.ico', 'image/x-icon'],
  ['.webp', 'image/webp'],
  ['.woff', 'font/woff'],
  ['.woff2', 'font/woff2'],
  ['.ttf', 'font/ttf'],
  ['.map', 'application/json; charset=utf-8'],
  ['.txt', 'text/plain; charset=utf-8'],
]);

function argument(name, fallback) {
  const index = process.argv.indexOf(`--${name}`);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

const dir = resolve(argument('dir', 'config-editor/dist'));
const port = Number(argument('port', '4173'));
const host = argument('host', '127.0.0.1');

if (!existsSync(dir) || !statSync(dir).isDirectory()) {
  console.error(`serve.mjs: directory not found: ${dir}`);
  process.exit(1);
}

function resolveFile(pathname) {
  const decoded = decodeURIComponent(pathname.split('?')[0]);
  const target = normalize(join(dir, decoded));
  if (target !== dir && !target.startsWith(dir + sep)) return null;
  let file = target;
  try {
    if (statSync(file).isDirectory()) file = join(file, 'index.html');
  } catch {
    return null;
  }
  return existsSync(file) && statSync(file).isFile() ? file : null;
}

const server = createServer((request, response) => {
  const file = resolveFile(request.url ?? '/');
  if (!file) {
    response.writeHead(404, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Not found');
    return;
  }
  response.writeHead(200, {
    'content-type': types.get(extname(file).toLowerCase()) ?? 'application/octet-stream',
    'cache-control': 'no-store',
  });
  createReadStream(file).pipe(response);
});

server.listen(port, host, () => {
  console.log(`Serving ${dir} at http://${host}:${port}/`);
});

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => server.close(() => process.exit(0)));
}
