# Recipes assume a POSIX shell: Unix keeps just's default `sh`, Windows uses Git Bash.
# Git Bash (`bash`) must be on PATH when running `just` on Windows.

set windows-shell := ["bash", "-c"]

# Pinned Zensical version, run on demand with `uvx`.
zensical := 'zensical==0.0.63'

# Show the available repository tasks.
default:
    @just --list

# Install Node.js dependencies.
setup:
    npm ci --prefix config-generator
    npm ci --prefix tests/config-generator

# Compile WASM, then start the config generator at http://127.0.0.1:8080/.
dev:
    npm run dev --prefix config-generator

# Start Vite without rebuilding WASM (for Svelte/CSS-only changes).
dev-ui:
    npm exec --prefix config-generator -- vite --host 127.0.0.1 --port 8080

# Rebuild WASM after Rust or XML changes while Vite is running.
wasm:
    npm run wasm --prefix config-generator

# Start the TUIC documentation development server.
docs:
    uvx '{{zensical}}' serve -f tuic/zensical.toml

# Start the Wind documentation development server.
docs-wind:
    uvx '{{zensical}}' serve -f wind/zensical.toml

# Run formatting, native/WASM Rust checks, tests, and Svelte checks.
check:
    cargo +nightly fmt --all --check
    cargo test --workspace --locked
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
    npm run check --prefix config-generator

# Build the TUIC documentation site into tuic/site/.
build-docs-tuic:
    uvx '{{zensical}}' build --clean -f tuic/zensical.toml

# Build the Wind documentation site into wind/site/.
build-docs-wind:
    uvx '{{zensical}}' build --clean -f wind/zensical.toml

# Build the standalone configuration generator for the /config-generator/ prefix.
build-generator:
    npm run build --prefix config-generator -- --base /config-generator/

# Build every documentation site and the standalone generator under site/.
build: build-docs-tuic build-docs-wind build-generator
    rm -rf site
    mkdir -p site
    cp -r tuic/site site/tuic
    cp -r wind/site site/wind
    cp -r config-generator/dist site/config-generator
    cp portal/index.html site/index.html

# Build and validate the assembled site.
site-check: build
    uvx python tests/config-generator/check-site.py

# Run browser regression tests against a temporary server for the generator dist/.
browser:
    uvx python tests/config-generator/run-browser.py

# Run browser regression tests against the assembled deployment build.
browser-site: build
    uvx python tests/config-generator/run-browser.py --directory site/config-generator --prefix /config-generator/

# Build, validate, and serve the assembled site preview.
preview: site-check
    uvx python tests/config-generator/preview-server.py
