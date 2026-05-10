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

function declarationPath(flavour, target) {
  return `${rawDir(flavour, target)}/cow_sdk_wasm.d.ts`;
}

function modulePath(flavour, target) {
  const extension = target === "nodejs" ? "cjs" : "js";
  return `${rawDir(flavour, target)}/cow_sdk_wasm.${extension}`;
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
  const typeTarget = preferredTypesTarget(flavour);
  const entry = {
    types: declarationPath(flavour, typeTarget)
  };

  if (flavour.targets.includes("nodejs")) {
    entry.node = {
      types: declarationPath(flavour, "nodejs"),
      require: modulePath(flavour, "nodejs"),
      default: modulePath(flavour, "nodejs")
    };
  }

  if (flavour.targets.includes("web")) {
    entry.browser = {
      types: declarationPath(flavour, "web"),
      import: modulePath(flavour, "web")
    };
  }

  if (flavour.targets.includes("bundler")) {
    entry.import = modulePath(flavour, "bundler");
    entry.default = modulePath(flavour, "bundler");
  } else {
    entry.import = modulePath(flavour, typeTarget);
    entry.default = modulePath(flavour, typeTarget);
  }

  return entry;
}

const defaultFlavour = descriptor.flavours.find((flavour) => flavour.subpath === ".");
if (!defaultFlavour) {
  throw new Error("flavours.json must define the default subpath");
}
requireTarget(defaultFlavour, "bundler");
requireTarget(defaultFlavour, "nodejs");

template.main = modulePath(defaultFlavour, "nodejs");
template.module = modulePath(defaultFlavour, "bundler");
template.types = declarationPath(defaultFlavour, "bundler");
template.exports = {};

for (const flavour of descriptor.flavours) {
  template.exports[flavour.subpath] = flavourExport(flavour);
  if (flavour.rawWasmSubpath) {
    template.exports[flavour.rawWasmSubpath] = wasmPath(flavour, "web");
  }
}

template.files = ["dist/", "README.md", "LICENSE"];

writeFileSync(packagePath, `${JSON.stringify(template, null, 2)}\n`);
