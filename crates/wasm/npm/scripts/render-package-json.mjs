import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const templatePath = join(packageRoot, "package.template.json");
const flavoursPath = join(packageRoot, "flavours.json");
const packagePath = join(packageRoot, "package.json");

const template = JSON.parse(readFileSync(templatePath, "utf8"));
const descriptor = JSON.parse(readFileSync(flavoursPath, "utf8"));
const packageName = process.env.NPM_PACKAGE_NAME?.trim();

if (packageName) {
  template.name = packageName;
}

function rawDir(flavour, target) {
  return `./dist/raw/${flavour.name}-${target}`;
}

function facadeDir(flavour) {
  return `./dist/${flavour.name}`;
}

function facadeDeclarationPath(flavour) {
  return `${facadeDir(flavour)}/index.d.ts`;
}

function facadeModulePath(flavour) {
  return `${facadeDir(flavour)}/index.mjs`;
}

function facadeRequirePath(flavour) {
  return `${facadeDir(flavour)}/index.cjs`;
}

function wasmPath(flavour, target) {
  return `${rawDir(flavour, target)}/cow_sdk_wasm_bg.wasm`;
}

function requireTarget(flavour, target) {
  if (!flavour.targets.includes(target)) {
    throw new Error(`${flavour.name} flavour does not define ${target} target`);
  }
}

function webFacadeModulePath(flavour) {
  return `${facadeDir(flavour)}/edge.mjs`;
}

function webFacadeDeclarationPath(flavour) {
  return `${facadeDir(flavour)}/edge.d.ts`;
}

function moduleFacadeModulePath(flavour) {
  return `${facadeDir(flavour)}/module.mjs`;
}

function moduleFacadeDeclarationPath(flavour) {
  return `${facadeDir(flavour)}/module.d.ts`;
}

function flavourExport(flavour) {
  const entry = {
    types: facadeDeclarationPath(flavour)
  };

  if (flavour.targets.includes("nodejs")) {
    entry.node = {
      types: facadeDeclarationPath(flavour),
      require: facadeRequirePath(flavour),
      default: facadeRequirePath(flavour)
    };
  }

  // When a flavour also ships the standalone `web` target, edge runtimes resolve
  // the explicit-init web build, where the host owns module instantiation. These
  // conditions precede `browser`/`import`/`default` so a Cloudflare Worker, Deno,
  // Vercel Edge, or Bun host matches the web entry rather than the bundler entry.
  if (flavour.targets.includes("web") && flavour.targets.includes("bundler")) {
    const web = {
      types: webFacadeDeclarationPath(flavour),
      default: webFacadeModulePath(flavour)
    };
    entry.workerd = web;
    entry.worker = web;
    entry.deno = web;
    entry["edge-light"] = web;
    entry.bun = web;
  }

  // A flavour that ships the standalone `web` target routes browser, import, and
  // default at the explicit-init web build (`edge.mjs`): its
  // `new URL(import.meta.url)` loader instantiates across every bundler and with no
  // bundler at all, where the bundler target's `import * as wasm` ESM integration is
  // not portable (it is broken on Rolldown/esbuild and webpack-first elsewhere). A
  // flavour with only `bundler`+`nodejs` keeps the bundler facade entry; browser
  // consumers are steered to a `web`-target flavour (`./trading`). The `node`
  // condition above keeps resolving to the nodejs CJS build, so Node is unaffected.
  const hasWeb = flavour.targets.includes("web");
  const browserModule = hasWeb ? webFacadeModulePath(flavour) : facadeModulePath(flavour);
  const browserTypes = hasWeb ? webFacadeDeclarationPath(flavour) : facadeDeclarationPath(flavour);

  if (hasWeb || flavour.targets.includes("bundler")) {
    entry.browser = {
      types: browserTypes,
      import: browserModule
    };
  }

  entry.import = browserModule;
  entry.default = browserModule;

  return entry;
}

const defaultFlavour = descriptor.flavours.find((flavour) => flavour.subpath === ".");
if (!defaultFlavour) {
  throw new Error("flavours.json must define the default subpath");
}
requireTarget(defaultFlavour, "bundler");
requireTarget(defaultFlavour, "nodejs");

template.main = facadeRequirePath(defaultFlavour);
// When the default flavour also ships the `web` target, the bundler ESM facade
// entry (`index.mjs`) is dropped (compile-facade.sh) — its browser/import/default
// conditions resolve to the portable web build (`edge.mjs`). Point the legacy
// top-level `module` field at that same web entry so it never references a
// pruned file. A default flavour without a web target keeps the bundler entry.
template.module = defaultFlavour.targets.includes("web")
  ? webFacadeModulePath(defaultFlavour)
  : facadeModulePath(defaultFlavour);
template.types = facadeDeclarationPath(defaultFlavour);
template.exports = {};

for (const flavour of descriptor.flavours) {
  template.exports[flavour.subpath] = flavourExport(flavour);
  if (flavour.webSubpath) {
    // The explicit web subpath (for example `./trading/edge`) is the unambiguous
    // entry for Cloudflare Workers, which import the precompiled module from the
    // `rawWasmSubpath` and pass it to `initialize`. Deno, Vercel Edge, and Bun can
    // use it too, or resolve the same build through the runtime conditions above.
    template.exports[flavour.webSubpath] = {
      types: webFacadeDeclarationPath(flavour),
      import: webFacadeModulePath(flavour),
      default: webFacadeModulePath(flavour)
    };
  }
  if (flavour.moduleSubpath) {
    // The standards-track `module` build (TC39 source-phase imports / Wasm ESM
    // Integration): `import source` + synchronous instantiation, auto-initializing
    // like the bundler build with no `initialize()` call. Supported today on
    // Node 24, Deno, and esbuild (external), and the forward path for browser
    // bundlers as source-phase lands. Opt-in subpath; the default `./trading`
    // browser path stays the `web` build until source-phase is broadly portable.
    template.exports[flavour.moduleSubpath] = {
      types: moduleFacadeDeclarationPath(flavour),
      import: moduleFacadeModulePath(flavour),
      default: moduleFacadeModulePath(flavour)
    };
  }
  if (flavour.rawWasmSubpath) {
    // The web and bundler targets emit a byte-identical wasm binary — only the JS
    // loader glue differs — so the raw Worker module subpath points at the bundler
    // copy and the redundant web binary is dropped (dedupe-target-wasm.mjs). The
    // web glue instantiates a module the host supplies, so it needs no sibling
    // binary. A flavour without a bundler target keeps the web binary as canonical.
    const rawWasmTarget = flavour.targets.includes("bundler") ? "bundler" : "web";
    template.exports[flavour.rawWasmSubpath] = wasmPath(flavour, rawWasmTarget);
  }
}

template.files = ["dist/", "README.md", "LICENSE"];

writeFileSync(packagePath, `${JSON.stringify(template, null, 2)}\n`);
