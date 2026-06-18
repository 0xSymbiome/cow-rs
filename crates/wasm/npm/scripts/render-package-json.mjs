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

function preferredTypesTarget(flavour) {
  if (flavour.targets.includes("bundler")) {
    return "bundler";
  }
  if (flavour.targets.includes("web")) {
    return "web";
  }
  return flavour.targets[0];
}

function webFacadeModulePath(flavour) {
  return `${facadeDir(flavour)}/edge.mjs`;
}

function webFacadeDeclarationPath(flavour) {
  return `${facadeDir(flavour)}/edge.d.ts`;
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

  // The browser condition points at the facade ESM entry (`index.mjs`), which is
  // independent of which raw wasm-bindgen target produced it. Emit it for any flavour
  // that has a browser-capable ESM build (`bundler` or `web`) so the published exports
  // map is unchanged when a flavour ships only `bundler`+`nodejs` (no standalone `web`).
  if (flavour.targets.includes("web") || flavour.targets.includes("bundler")) {
    entry.browser = {
      types: facadeDeclarationPath(flavour),
      import: facadeModulePath(flavour)
    };
  }

  preferredTypesTarget(flavour);
  entry.import = facadeModulePath(flavour);
  entry.default = facadeModulePath(flavour);

  return entry;
}

const defaultFlavour = descriptor.flavours.find((flavour) => flavour.subpath === ".");
if (!defaultFlavour) {
  throw new Error("flavours.json must define the default subpath");
}
requireTarget(defaultFlavour, "bundler");
requireTarget(defaultFlavour, "nodejs");

template.main = facadeRequirePath(defaultFlavour);
template.module = facadeModulePath(defaultFlavour);
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
