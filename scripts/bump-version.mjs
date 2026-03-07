#!/usr/bin/env node
import fs from "fs";
import path from "path";
import { execSync } from "child_process";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");

const version = process.argv[2];
if (!version) {
  console.error("Usage: node scripts/bump-version.mjs <version>");
  console.error("Example: node scripts/bump-version.mjs 1.0.0");
  process.exit(1);
}

// バージョン形式の検証
if (!/^\d+\.\d+\.\d+(-[\w.]+)?$/.test(version)) {
  console.error("Invalid version format. Use semver (e.g., 1.0.0, 1.0.0-beta.1)");
  process.exit(1);
}

// package.json
const packageJsonPath = path.join(rootDir, "package.json");
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8"));
packageJson.version = version;
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
console.log(`Updated package.json to ${version}`);

// src-tauri/tauri.conf.json
const tauriConfPath = path.join(rootDir, "src-tauri", "tauri.conf.json");
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, "utf-8"));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + "\n");
console.log(`Updated tauri.conf.json to ${version}`);

// src-tauri/Cargo.toml
const cargoTomlPath = path.join(rootDir, "src-tauri", "Cargo.toml");
let cargoToml = fs.readFileSync(cargoTomlPath, "utf-8");
cargoToml = cargoToml.replace(/^version = ".*"$/m, `version = "${version}"`);
fs.writeFileSync(cargoTomlPath, cargoToml);
console.log(`Updated Cargo.toml to ${version}`);

// Cargo.lock（cargo updateでCargo.tomlの変更を反映）
execSync("cargo update --workspace --manifest-path src-tauri/Cargo.toml", {
  cwd: rootDir,
  stdio: "inherit",
});
console.log(`Updated Cargo.lock to ${version}`);

// .claude-plugin/plugin.json
const pluginJsonPath = path.join(rootDir, ".claude-plugin", "plugin.json");
const pluginJson = JSON.parse(fs.readFileSync(pluginJsonPath, "utf-8"));
pluginJson.version = version;
fs.writeFileSync(pluginJsonPath, JSON.stringify(pluginJson, null, 2) + "\n");
console.log(`Updated .claude-plugin/plugin.json to ${version}`);

// .claude-plugin/marketplace.json
const marketplaceJsonPath = path.join(rootDir, ".claude-plugin", "marketplace.json");
const marketplaceJson = JSON.parse(fs.readFileSync(marketplaceJsonPath, "utf-8"));
marketplaceJson.plugins[0].version = version;
fs.writeFileSync(marketplaceJsonPath, JSON.stringify(marketplaceJson, null, 2) + "\n");
console.log(`Updated .claude-plugin/marketplace.json to ${version}`);

// CHANGELOG.md を git-cliff で更新
const tag = `v${version}`;
const changelogPath = path.join(rootDir, "CHANGELOG.md");
if (!fs.existsSync(changelogPath)) {
  fs.writeFileSync(changelogPath, "");
}
try {
  execSync(`git cliff --unreleased --tag ${tag} --prepend CHANGELOG.md`, {
    cwd: rootDir,
    stdio: "inherit",
  });
  console.log(`Updated CHANGELOG.md for ${tag}`);
} catch {
  console.error("git-cliff failed. Install with: cargo install git-cliff");
  process.exit(1);
}

console.log(`\nAll files updated to version ${version}`);
