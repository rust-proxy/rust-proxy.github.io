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
    npm ci --prefix config-editor
    npm ci --prefix tests/config-editor

# Compile WASM, then start the config editor at http://127.0.0.1:8080/.
dev:
    npm run dev --prefix config-editor

# Start Vite without rebuilding WASM (for Svelte/CSS-only changes).
dev-ui:
    npm exec --prefix config-editor -- vite --host 127.0.0.1 --port 8080

# Rebuild WASM after Rust or XML changes while Vite is running.
wasm:
    npm run wasm --prefix config-editor

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
    npm run check --prefix config-editor
    npm run test --prefix config-editor

# Build the TUIC documentation site into tuic/site/.
build-docs-tuic:
    uvx '{{zensical}}' build --clean -f tuic/zensical.toml

# Build the Wind documentation site into wind/site/.
build-docs-wind:
    uvx '{{zensical}}' build --clean -f wind/zensical.toml

# Build the standalone configuration editor for the /config-editor/ prefix.
build-editor:
    npm run build --prefix config-editor -- --base /config-editor/

# Build every documentation site and the standalone editor under site/.
build: build-docs-tuic build-docs-wind build-editor
    rm -rf site
    mkdir -p site
    cp -r tuic/site site/tuic
    cp -r wind/site site/wind
    cp -r config-editor/dist site/config-editor
    cp portal/index.html site/index.html

# Build and validate the assembled site.
site-check: build
    uvx python tests/config-editor/check-site.py

# Run browser regression tests against a temporary server for the editor dist/.
browser:
    uvx python tests/config-editor/run-browser.py

# Run browser regression tests against the assembled deployment build.
browser-site: build
    uvx python tests/config-editor/run-browser.py --directory site/config-editor --prefix /config-editor/

# Build, validate, and serve the assembled site preview.
preview: site-check
    uvx python tests/config-editor/preview-server.py
