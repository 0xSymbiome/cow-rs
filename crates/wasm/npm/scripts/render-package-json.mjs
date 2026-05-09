import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const templatePath = join(packageRoot, "package.template.json");
const packagePath = join(packageRoot, "package.json");

const template = JSON.parse(readFileSync(templatePath, "utf8"));
const packageName = process.env.NPM_PACKAGE_NAME?.trim();
if (packageName) {
  template.name = packageName;
}

const denoDistExists = existsSync(join(packageRoot, "dist", "deno", "cow_sdk_wasm.js"));
if (process.env.BUILD_DENO === "1" && denoDistExists) {
  template.exports["./deno"] = {
    types: "./dist/deno/cow_sdk_wasm.d.ts",
    import: "./dist/deno/cow_sdk_wasm.js"
  };
}

writeFileSync(packagePath, `${JSON.stringify(template, null, 2)}\n`);
