#!/usr/bin/env node
import { execSync } from "child_process";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import path from "path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");

const { version } = JSON.parse(readFileSync(path.join(rootDir, "package.json"), "utf-8"));
const tag = `v${version}`;

try {
  execSync(`git tag ${tag}`, { cwd: rootDir, stdio: "inherit" });
  console.log(`Created tag: ${tag}`);
  console.log(`Next: git push origin main --follow-tags`);
} catch {
  console.error(`Failed to create tag ${tag}. Already exists?`);
  console.error(`To delete and recreate: git tag -d ${tag} && pnpm tag`);
  process.exit(1);
}
