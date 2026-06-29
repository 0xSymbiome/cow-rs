import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const packagePath = join(packageRoot, "package.json");
const flavoursPath = join(packageRoot, "flavours.json");

function fail(message) {
  console.error(`verify-exports: ${message}`);
  process.exitCode = 1;
}

function collectTargets(value, output = []) {
  if (typeof value === "string") {
    output.push(value);
    return output;
  }
  if (value && typeof value === "object") {
    for (const nested of Object.values(value)) {
      collectTargets(nested, output);
    }
  }
  return output;
}

function collectFiles(directory, output = []) {
  if (!existsSync(directory)) {
    return output;
  }

  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      collectFiles(path, output);
    } else {
      output.push(path);
    }
  }
  return output;
}

if (!existsSync(packagePath)) {
  fail("package.json has not been rendered");
  process.exit();
}

const manifest = JSON.parse(readFileSync(packagePath, "utf8"));
const descriptor = JSON.parse(readFileSync(flavoursPath, "utf8"));
const expectedExports = new Set(
  descriptor.flavours.flatMap((flavour) => {
    const subpaths = [flavour.subpath];
    if (flavour.webSubpath) {
      subpaths.push(flavour.webSubpath);
    }
    if (flavour.rawWasmSubpath) {
      subpaths.push(flavour.rawWasmSubpath);
    }
    if (flavour.moduleSubpath) {
      subpaths.push(flavour.moduleSubpath);
    }
    return subpaths;
  })
);

for (const forbidden of ["./web", "./bundler", "./nodejs"]) {
  if (Object.hasOwn(manifest.exports, forbidden)) {
    fail(`raw wasm-pack target subpath must not be exported: ${forbidden}`);
  }
}

for (const key of Object.keys(manifest.exports)) {
  if (!expectedExports.has(key)) {
    fail(`unexpected package export: ${key}`);
  }
}

for (const key of expectedExports) {
  if (!Object.hasOwn(manifest.exports, key)) {
    fail(`missing package export: ${key}`);
  }
}

const targets = [
  manifest.main,
  manifest.module,
  manifest.types,
  ...collectTargets(manifest.exports)
].filter(Boolean);

for (const target of targets) {
  if (!target.startsWith("./")) {
    fail(`export target must be package-relative: ${target}`);
    continue;
  }
  const path = join(packageRoot, target);
  if (!existsSync(path)) {
    fail(`missing export target: ${target}`);
    continue;
  }

  if (target.endsWith(".d.ts")) {
    const declaration = readFileSync(path, "utf8");
    if (
      declaration.includes("[Symbol.dispose]") &&
      !declaration.includes('reference lib="esnext.disposable"')
    ) {
      fail(`missing esnext.disposable reference in ${target}`);
    }
  }
}

for (const path of collectFiles(join(packageRoot, "dist"))) {
  if (path.endsWith(`${sep}README.md`) || path.endsWith(`${sep}package.json`)) {
    fail(`generated dist metadata must not be published: ${path.slice(packageRoot.length + 1)}`);
  }
}

// The export-map check above proves each entry file exists, but not that the entry
// resolves its own `./...` imports — each facade entry binds a raw shim through a
// relative specifier, so a generated or renamed shim an entry references but the
// build never emitted (a missing `<flavour>-web.js`, say) would pass every check
// above and only fail at a consumer's bundler. Assert every relative import each
// shipped facade entry declares resolves to a shipped file.
const relativeImport = /\bfrom\s+["'](\.[^"']+)["']|\brequire\(["'](\.[^"']+)["']\)/g;
for (const flavour of descriptor.flavours) {
  const flavourDir = join(packageRoot, "dist", flavour.name);
  if (!existsSync(flavourDir)) {
    continue;
  }
  for (const entry of ["index.mjs", "index.cjs", "edge.mjs", "module.mjs"]) {
    const entryPath = join(flavourDir, entry);
    if (!existsSync(entryPath)) {
      continue;
    }
    for (const match of readFileSync(entryPath, "utf8").matchAll(relativeImport)) {
      const specifier = match[1] ?? match[2];
      if (!existsSync(join(flavourDir, specifier))) {
        fail(`${flavour.name}/${entry} imports ${specifier}, which is not a shipped file`);
      }
    }
  }
}

if (process.exitCode) {
  process.exit(process.exitCode);
}

console.log(`verify-exports: checked ${targets.length} package targets`);
