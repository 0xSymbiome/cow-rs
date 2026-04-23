# WASM Examples

Standalone WASM examples for `cow-rs`.

If you are new to `cow-rs`, start with
[Getting Started](../../docs/getting-started.md) first. Come to the WASM
surfaces after the deterministic native onboarding flow is clear or when you
specifically need browser-hosted verification.

## Surfaces

| Console | Package | Purpose |
| --- | --- | --- |
| `sdk-verification-console/` | `cow-sdk-verification-console` | Deterministic SDK verification and browser inspection for WASM-compatible surfaces |
| `browser-wallet-console/` | `cow-sdk-browser-wallet-console-wasm` | Mock-wallet proof plus explicit injected-wallet flows for browser-runtime support |

`cow-sdk` remains the default facade for pure SDK flows. Browser-wallet support
is additive behind the `browser-wallet` feature and is intentionally separated
from the deterministic native onboarding path.

Each console is self-contained and served from its own directory. The palette,
layout, and polish primitives live inline in each console's `index.html` behind
a `shared polish primitives — keep synchronized across consoles` comment marker
so the rendered surface works unchanged under every serve flow (per-console
`bunx serve`, `python -m http.server`, and the Playwright `serve:console`
command). When both consoles need the same primitive, update the marker block
in both files in the same change so the two consoles stay visually identical
on the polish surface.

## Local Use

Build and serve each example from its own directory:

```text
cd examples/wasm/sdk-verification-console
wasm-pack build --target web
bunx serve . --listen 8080
```

```text
cd examples/wasm/browser-wallet-console
wasm-pack build --target web
bunx serve . --listen 8081
```

Open the served HTTP URL. Browsers do not load the generated WASM modules from
`file://` origins.

## Validation

SDK verification console:

```text
cd examples/wasm/sdk-verification-console
wasm-pack test --headless --chrome
```

Browser-wallet console:

```text
bun run --cwd e2e/browser-wallet test
```

## GitHub Pages

Published paths:

```text
https://<owner>.github.io/<repo>/sdk-verification-console/
https://<owner>.github.io/<repo>/browser-wallet-console/
```

## Current publication posture

The WASM consoles in this directory carry footer links that reference the
hosted-build authority where the current Pages deploy runs. The workspace crate
metadata in `Cargo.toml::repository` names the publication authority, and
`SECURITY.md` names the publication authority for private advisory reports.
These two owners are intentionally distinct during the current
pre-publication posture:

- Hosted build authority: where the rendered consoles actually run today.
- Publication authority: where the published crates and the security advisory
  report live.

When the Pages deploy rotates to the publication authority, both consoles'
footer URLs will move at the same time and the dual-authority acknowledgement
comment at the top of each `index.html` will be removed. The direct path to
that state is rotating the Pages deploy so every surface names the publication
authority. An alternative future posture is a build-time owner substitution
through a `{{owner}}` placeholder in the `index.html` template that the
publishing step resolves before the artifact ships; that path is not currently
implemented.

`scripts/check-release-docs-agree.sh` asserts the acknowledgement sentinel is
present on every `examples/wasm/*/index.html` so the dual-authority state
cannot drift silently. Once the rotation completes and the comments are
removed, the drift lint drops with them.
