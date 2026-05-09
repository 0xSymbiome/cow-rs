import { existsSync, readdirSync, readFileSync } from "node:fs";
import { dirname, join, sep } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const packagePath = join(packageRoot, "package.json");

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

if (process.exitCode) {
  process.exit(process.exitCode);
}

console.log(`verify-exports: checked ${targets.length} package targets`);
