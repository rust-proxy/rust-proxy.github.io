# rust-proxy Documentation Sites

This repository is the organization Pages repository `rust-proxy.github.io`, whose Pages site root is `https://rust-proxy.github.io/`. Each project is an independent static site under its own subpath; the root provides a portal page.

- TUIC documentation: [rust-proxy.github.io/tuic](https://rust-proxy.github.io/tuic/).
- Configuration generator (renders TUIC configuration by default): [rust-proxy.github.io/config-generator/](https://rust-proxy.github.io/config-generator/). It has its own HTML, CSS, WebAssembly, and theme, and runs standalone on any static server without depending on a documentation site or backend.
- Wind documentation: [rust-proxy.github.io/wind](https://rust-proxy.github.io/wind/), with English specifications at [rust-proxy.github.io/wind/specs](https://rust-proxy.github.io/wind/specs/).

The sites are built with [Zensical](https://zensical.org/docs/); the configuration generator is built with **Rust WASM + Svelte 5**.

## Running the configuration generator standalone

Requires Rust stable, the `wasm32-unknown-unknown` target, and Node.js 22.12+ (CI uses 24). Run all of the following commands from this repository's root:

```sh
rustup target add wasm32-unknown-unknown
npm ci --prefix config-generator
npm run dev --prefix config-generator
```

Open `http://127.0.0.1:8080/`. No Python or Zensical server is needed. It supports paired generation, server-only or client-only generation, multiple users, three certificate modes, SOCKS5 authentication, logging, connections, TCP/UDP forwarding, Quinn/quiche backends, outbound and ACL routing, DNS/GeoData, RESTful management, and HTTP/3 masquerading; "configuration details" browses the YAML by enum branch and shows per-field explanations on hover or keyboard focus.

When [just](https://just.systems/) is installed, the repository root provides shortcuts:

```sh
just setup   # install locked dependencies for the first time
just dev     # compile WASM and start the generator
just dev-ui  # skip the WASM build when only changing Svelte/CSS
just wasm    # recompile Rust/XML while Vite is running
just check   # Rust, WASM, and Svelte checks
```

Run `just` to see all build, preview, and browser-regression commands.

Standalone build:

```sh
npm run build --prefix config-generator
```

`npm run build` compiles the Rust library with the locked wasm-pack, runs the Svelte/TypeScript checks, and then bundles the local JS/CSS/WASM with Vite. The first build downloads the wasm-bindgen tool matching the Cargo lock file. `npm run dev` compiles WASM first and then starts Vite; Svelte/CSS supports hot reloading. After changing Rust or XML, run `npm run wasm --prefix config-generator` in another terminal and refresh the browser.

`CONFIG_SCHEMA` selects a different XML; paths are relative to `config-generator/` (or absolute), defaulting to `schema/config.xml`. Alternative builds should output to a separate directory, and the environment and default WASM must be restored afterward; see [DSL v4](tuic/docs/tools/config-dsl.md).

Artifacts live in `config-generator/dist/`. Hand the entire directory to a static server; it uses relative asset paths by default and supports either the root path or a subpath with a trailing `/`. The server must return `application/wasm` for `.wasm`; do not open the files over `file://`. For a fixed prefix, use `npm run build --prefix config-generator -- --base /your-prefix/`.

Credentials are generated with the browser Crypto API. All input, validation, and serialization happen locally in WASM; no third-party analytics scripts are loaded, inputs, themes, and credentials are not saved, and configuration is never submitted over the network. Copy and download include plaintext passwords, while the preview hides passwords by default.

The page accepts `scheme` and `mode` query parameters for direct links. `scheme` is a configuration name shared by a top-level output and a `<config-desc>` configuration (for example `server` or `client`); `mode` is `generate` or `detail`. The configuration selector and page tabs keep these parameters synchronized without discarding unrelated query parameters or the URL fragment.

## Running the documentation locally

Requires [uv](https://docs.astral.sh/uv/); `uvx` fetches the pinned Zensical and Python on demand, so no project virtual environment is created.

```sh
just docs
just docs-wind
```

Each documentation site has its own `zensical.toml` and `docs/`; the development server serves only one site at a time, and the configuration generator runs separately with the Vite server above. When adding a documentation site, create `<project>/zensical.toml` and `<project>/docs/`, and add it to the `build` recipe in `justfile` and to the portal links.

## Combined build and validation

```sh
# Rust core logic, DSL, configuration, and security boundaries
cargo test --workspace --locked
cargo +nightly fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --target wasm32-unknown-unknown --lib --locked -- -D warnings

# Check Svelte/TypeScript after building WASM (build also runs this)
npm run check --prefix config-generator

# Build all documentation sites and the standalone generator into site/; no publishing
just build
uvx python tests/config-generator/check-site.py

# Assemble a site preview (/, /tuic/, /wind/)
uvx python tests/config-generator/preview-server.py
```

Preview: `http://127.0.0.1:8765/`, `http://127.0.0.1:8765/tuic/`, `http://127.0.0.1:8765/config-generator/`, `http://127.0.0.1:8765/wind/`. Run `npm ci --prefix config-generator` first to install the frontend dependencies; the combined build clean-builds each documentation site, places the standalone generator under `site/config-generator/`, and copies `portal/index.html` to `site/index.html`. Stop the documentation development servers before a clean build to avoid cache conflicts.

### Standalone format parsing and real TUIC checks

```sh
# Generate test credentials and configuration dynamically; writes only to the ignored .cache/
cargo run --locked --example fixtures -- .cache/config-generator-fixtures

# Python 3.11+ independent parsers for tomllib, json, and PyYAML
uvx --with 'PyYAML>=6,<7' python tests/config-generator/roundtrip.py .cache/config-generator-fixtures/roundtrip.json

# Call the neighboring TUIC's real parsing functions and run a local SOCKS5 -> TUIC -> TCP echo
uvx python tests/config-generator/check-rust.py --offline
```

The real parsing check requires the neighboring `../tuic`, its submodules, cached dependencies, and the corresponding build tools; omit `--offline` when the dependency cache is missing. The auxiliary Cargo project writes only to `.cache/`, starts from TUIC's lock file, and does not modify TUIC manifests, sources, lock files, or submodules. The loopback allowance applies only to the in-memory test configuration, and error diagnostics never print configuration contents.

### Browser checks

The browser tests use a separate npm manifest and lock file; test dependencies are not bundled into the application. Start the assembled preview above, then run (locally this uses the installed Edge by default):

```sh
npm ci --prefix tests/config-generator
node tests/config-generator/browser.mjs

# Test the built standalone artifact; the temporary local server shuts down with the test
uvx python tests/config-generator/run-browser.py

# Use the deployment prefix for the assembled site build
uvx python tests/config-generator/run-browser.py --directory site/config-generator --prefix /config-generator/

# Alternatively use Playwright's bundled Chromium, matching CI
npm exec --prefix tests/config-generator -- playwright install chromium
BROWSER_CHANNEL=chromium node tests/config-generator/browser.mjs
```

`PLAYWRIGHT_MODULE_PATH` can point to an existing Playwright module directory; `BROWSER_CHANNEL` accepts `msedge`, `chrome`, and `chromium`, with CI defaulting to `chromium`. The standalone Vite development server uses `PREVIEW_URL=http://127.0.0.1:8080/`. The tests cover WASM loading, standalone page structure, pairing consistency, user removal, TLS switching, input validation, forwarding edits, copy/download, escaping, mobile, theming, no external requests, and no input persistence; screenshots go to `.cache/`. The reuse check builds with `schema/example.xml` into `.cache/generic-site` and runs `uvx python tests/config-generator/run-browser.py --directory .cache/generic-site --script tests/config-generator/browser-generic.mjs`; see the DSL documentation for the full command. CI likewise keeps the default site artifacts for later publishing.

## DSL and maintenance conventions

[Config DSL v4](tuic/docs/tools/config-dsl.md) uses a separate `config-generator/schema/config.xml` to statically describe inputs, defaults, enums, conditions, lists, mappings, and sensitive fields, deserialized with quick-xml + Serde; configuration descriptions are not written with Rust macros or closures. The Rust session produces the form view, and Svelte renders it; generic projection and redaction live in `dsl.rs`, generic validation and field linkage in `dsl/rules.rs`, and basic address checks in `validation.rs`. TUIC branding, page sections, hints, cross-field rules, random-value generation declarations, and export commands are also entirely provided by the XML. Adding a target application only requires swapping the XML; `schema/example.xml` provides a reuse example with no TUIC fields.

Configuration state is modified only by the Rust `Session`. Svelte submits generic field/collection operations and reads field display values, visibility, errors, and previews from `Snapshot`; it does not parse XML, evaluate conditions, or keep a second mutable copy of the configuration. JSON strings cross the WASM boundary, and both field display values and stable row identities are strings, avoiding JavaScript number precision loss. `ui/types.ts` corresponds to the display contract in `session/view.rs`; when changing the contract, update both sides and run the session tests plus both browser test suites. Copy and download obtain the original text through a separate `export` operation rather than reading the redacted preview.

| Path | Contents |
| --- | --- |
| `Cargo.toml` / `Cargo.lock` | Generator Rust workspace and locked dependencies |
| `tuic/zensical.toml` / `tuic/docs/` | TUIC Chinese documentation, navigation, field descriptions, and DSL documentation |
| `tuic/overrides/` | TUIC theme overrides and 404 page |
| `config-generator/` | Independently buildable Rust WASM + Svelte single-page application |
| `config-generator/ui/` | Generic Svelte controls, page layout, browser operations, and display contract |
| `config-generator/src/session.rs` / `session/view.rs` | Natively testable editing operations, form view, and preview export |
| `config-generator/src/wasm.rs` | WASM interface and browser Crypto API randomness adapter |
| `config-generator/package.json` / `vite.config.js` | Locked frontend tooling and static asset bundling |
| `config-generator/schema/config.xml` | The single TUIC product definition: UI, fields, rules, hints, and output |
| `config-generator/src/dsl/xml.rs` / `dsl/wire.rs` / `dsl/parser.rs` | XML subset checks, Serde data model, and semantic validation |
| `config-generator/src/dsl.rs` | Data projection, type checking, and redaction |
| `config-generator/src/schema.rs` | Embedded XML, cached parse results, generic state, and stable row identities |
| `config-generator/src/dsl/metadata.rs` / `dsl/rules.rs` | Page metadata, random-value declarations, validation, and field linkage |
| `config-generator/schema/example.xml` | Complete application reuse example with no TUIC fields |
| `config-generator/src/model.rs` | Configuration generation entry point and three-format serialization |
| `config-generator/tests/` | XML DSL and configuration regression tests |
| `wind/zensical.toml` / `wind/docs/` | Wind Chinese protocol specifications and design documents (single publishing source) |
| `wind/docs/specs/` | Wind English specifications and RFC template, published under `/wind/specs/` and kept in sync with the Chinese editions |
| `portal/index.html` | Site root portal page |
| `justfile` | Build, check, and preview recipes; the `build` recipe assembles all documentation sites and the generator into `site/` without publishing |
| `tests/config-generator/` | Standalone parser, real TUIC, site, and browser checks |
| `.github/workflows/deploy.yml` | GitHub Pages build and publish workflow |

The main documentation is maintained only in Simplified Chinese; the English specifications and RFC template under `wind/docs/specs/` are the exception, published alongside their Chinese counterparts under `/wind/specs/`, with section numbering and requirements kept in sync. After updating TUIC or Wind, verify the generator's version baseline and the actual runtime behavior of fields, and do not expose configuration that is not yet wired into client runtime logic as usable functionality. Examples use placeholder domains and test credentials generated at runtime, and do not include real deployment data.

## Publishing paths

The [CI and Pages workflow](.github/workflows/deploy.yml) runs on pull requests, pushes to `main`, and manual triggers:

- `check`: nightly rustfmt, stable native and WASM Clippy, Rust/XML DSL tests, and independent TOML/JSON/YAML parsing round trips. Python is pinned to 3.13 via `UV_PYTHON`, and tools run on demand with `uvx`.
- `build`: builds the standalone SPA with wasm-pack, the Svelte checker, and Vite, assembles all documentation sites, checks site links and assets, and then runs the TUIC and no-TUIC-field XML reuse browser regressions through the locked Playwright/Chromium. Rust, uv, and npm use dependency caching.
- `deploy`: depends on `check` and `build` succeeding, and publishes only on pushes to `main` or manual runs; Pages write and OIDC permissions are granted only to this job, while pull requests only validate and build.

The published artifact is assembled in a temporary directory whose root is `https://rust-proxy.github.io/`: the portal page is at `/`, TUIC documentation at `/tuic/`, the standalone generator at `/config-generator/`, Wind Chinese documentation at `/wind/`, and the Wind English specifications at `/wind/specs/`. No custom domain or `CNAME` is used, and the Pages source should be set to GitHub Actions. Real TUIC parsing and loopback tests still run in an environment with the neighboring repositories as described above.

The documentation has migrated from MkDocs to Zensical and no longer uses the i18n plugin. The old `/tuic/zh/` path does not generate a redirect; external links should be updated under `/tuic/`. A passing site build and configuration parse does not mean remote DNS, certificates, firewalls, or proxy connections have been verified.
