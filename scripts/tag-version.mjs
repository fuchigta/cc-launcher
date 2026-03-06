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
} catch {
  console.error(`Failed to create tag ${tag}. Already exists?`);
  console.error(`To delete and recreate: git tag -d ${tag} && pnpm tag`);
  process.exit(1);
}

try {
  execSync(`git push origin main --follow-tags`, { cwd: rootDir, stdio: "inherit" });
  console.log(`Pushed main and ${tag} to origin`);
} catch {
  console.error(`Failed to push. Run manually: git push origin main --follow-tags`);
  process.exit(1);
}
