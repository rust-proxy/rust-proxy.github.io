"""Assemble each documentation site and the standalone SPA under site/. No publishing."""
from pathlib import Path
import shutil
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
npm = shutil.which('npm.cmd') or shutil.which('npm')
if not npm:
    raise SystemExit('Node.js/npm is required; run npm ci --prefix config-generator first')

sites = ('tuic', 'wind')
output = root / 'site'
shutil.rmtree(output, ignore_errors=True)
output.mkdir(parents=True)

for site in sites:
    subprocess.run([sys.executable, '-m', 'zensical', 'build', '--clean', '-f', f'{site}/zensical.toml'],
                   cwd=root, check=True)
    shutil.copytree(root / site / 'site', output / site, dirs_exist_ok=True)

subprocess.run([npm, 'run', 'build', '--prefix', 'config-generator', '--', '--base', '/config-generator/'],
               cwd=root, check=True)
shutil.copytree(root / 'config-generator/dist', output / 'config-generator', dirs_exist_ok=True)

shutil.copyfile(root / 'portal/index.html', output / 'index.html')
print('Built documentation sites and standalone SPA under site/; nothing published')
