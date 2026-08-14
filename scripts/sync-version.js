// Synchronizes the root package.json version to apps/desktop/src-tauri/Cargo.toml.
// Run via `pnpm version:sync` after `pnpm version patch|minor|major`.

const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const packageJsonPath = path.join(root, 'package.json');
const cargoPath = path.join(root, 'apps/desktop/src-tauri/Cargo.toml');

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

if (!version) {
  console.error('No version found in package.json');
  process.exit(1);
}

let cargoContent = fs.readFileSync(cargoPath, 'utf8');
const versionPattern = /^version = ".*"$/m;
if (!versionPattern.test(cargoContent)) {
  console.error('Could not find version field in Cargo.toml');
  process.exit(1);
}

cargoContent = cargoContent.replace(versionPattern, `version = "${version}"`);
fs.writeFileSync(cargoPath, cargoContent, 'utf8');

console.log(`Synced version ${version} to apps/desktop/src-tauri/Cargo.toml`);
