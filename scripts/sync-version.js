// Synchronizes the root package.json version to every file that must agree
// on it: the Cargo workspace version, the Tauri bundle version, the landing
// site constants, and every workspace member's package.json. Run via
// `pnpm version:sync` after any `pnpm version:*` bump (the version:pre:*
// scripts call it too).

const fs = require('node:fs');
const path = require('node:path');

const root = path.resolve(__dirname, '..');
const packageJsonPath = path.join(root, 'package.json');
const cargoPath = path.join(root, 'Cargo.toml');
const tauriConfPath = path.join(root, 'apps/desktop/src-tauri/tauri.conf.json');
const siteTsPath = path.join(root, 'apps/landing/src/lib/site.ts');
const memberPackagePaths = [
  'apps/desktop/package.json',
  'apps/landing/package.json',
  'apps/mobile/package.json',
  'apps/pairing-server/package.json',
  'packages/core/package.json',
  'packages/i18n/package.json',
  'packages/ui/package.json',
];

const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
const version = packageJson.version;

if (!version) {
  console.error('No version found in package.json');
  process.exit(1);
}

// Cargo: apps/desktop/src-tauri/Cargo.toml inherits via `version.workspace =
// true`, so the root [workspace.package] block is the only Rust source.
let cargo = fs.readFileSync(cargoPath, 'utf8');
let inWorkspacePackage = false;
let cargoReplaced = false;
cargo = cargo
  .split('\n')
  .map((line) => {
    if (line.startsWith('[')) {
      inWorkspacePackage = line.trim() === '[workspace.package]';
      return line;
    }
    if (inWorkspacePackage && !cargoReplaced && /^version\s*=/.test(line)) {
      cargoReplaced = true;
      return `version = "${version}"`;
    }
    return line;
  })
  .join('\n');
if (!cargoReplaced) {
  console.error('Could not find version field under [workspace.package] in root Cargo.toml');
  process.exit(1);
}
fs.writeFileSync(cargoPath, cargo, 'utf8');

// Tauri bundle version (what the updater compares against).
const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
tauriConf.version = version;
fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');

// Landing site constants (SITE.version — the "current version" fact line).
let siteTs = fs.readFileSync(siteTsPath, 'utf8');
const sitePattern = /version: '[^']*'/;
if (!sitePattern.test(siteTs)) {
  console.error(`Could not find version field in ${path.relative(root, siteTsPath)}`);
  process.exit(1);
}
siteTs = siteTs.replace(sitePattern, `version: '${version}'`);
fs.writeFileSync(siteTsPath, siteTs, 'utf8');

// Every workspace member's package.json (published metadata must agree).
let syncedMembers = 0;
for (const relPath of memberPackagePaths) {
  const absPath = path.join(root, relPath);
  const pkg = JSON.parse(fs.readFileSync(absPath, 'utf8'));
  if (pkg.version === version) continue;
  pkg.version = version;
  fs.writeFileSync(absPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
  syncedMembers += 1;
}

console.log(
  `Synced version ${version} to: root Cargo.toml [workspace.package], tauri.conf.json, landing site.ts, ${syncedMembers} member package.json`
);
