#!/usr/bin/env node
import { readFileSync, writeFileSync } from "fs";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(__dirname, "..");

const packageJsonPath = resolve(rootDir, "package.json");
const cargoTomlPath = resolve(rootDir, "src-tauri", "Cargo.toml");

const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf-8"));
const version = packageJson?.version;

if (typeof version !== "string" || version.length === 0) {
  throw new Error("package.json version is missing or invalid");
}

const cargoToml = readFileSync(cargoTomlPath, "utf-8");
const updatedCargoToml = cargoToml.replace(
  /^version\s*=\s*"[^"]*"/m,
  `version = "${version}"`
);

if (updatedCargoToml === cargoToml) {
  console.log(`Cargo.toml already matches version ${version}`);
  process.exit(0);
}

writeFileSync(cargoTomlPath, updatedCargoToml);
console.log(`Synced version ${version} to Cargo.toml`);
