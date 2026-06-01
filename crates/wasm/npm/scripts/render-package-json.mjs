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
  if (flavour.rawWasmSubpath) {
    template.exports[flavour.rawWasmSubpath] = wasmPath(flavour, "web");
  }
}

template.files = ["dist/", "README.md", "LICENSE"];

writeFileSync(packagePath, `${JSON.stringify(template, null, 2)}\n`);
