# Use the native shell on Windows; Unix keeps just's default `sh` shell.

set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

# Show the available repository tasks.
default:
    @just --list

# Install locked Python and Node.js dependencies.
setup:
    uv sync --locked
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

# Start the documentation development server.
docs:
    uv run --locked zensical serve

# Run formatting, native/WASM Rust checks, tests, and Svelte checks.
check:
    cargo +nightly fmt --all --check
    cargo test --workspace --locked
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings
    npm run check --prefix config-generator

# Build the documentation and standalone generator under site/.
build:
    uv run --locked python scripts/build-site.py

# Build and validate the combined site.
site-check: build
    uv run --locked python tests/config-generator/check-site.py

# Run browser regression tests against a temporary server for config-generator/dist/.
browser:
    uv run --locked python tests/config-generator/run-browser.py

# Run browser regression tests against the combined deployment-prefix build.
browser-site: build
    uv run --locked python tests/config-generator/run-browser.py --directory site/config-generator --prefix /tuic/config-generator/

# Build, validate, and serve the combined site preview.
preview: site-check
    uv run --locked python tests/config-generator/preview-server.py
