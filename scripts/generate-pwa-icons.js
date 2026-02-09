#!/usr/bin/env node
import fs from "fs";
import path from "path";
import sharp from "sharp";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

const candidates = [
  path.join(rootDir, "app-icon.png"),
  path.join(rootDir, "public", "app-icon.png"),
];
const sourceIcon = candidates.find((p) => fs.existsSync(p));

if (!sourceIcon) {
  throw new Error("Missing source icon: app-icon.png");
}

const outputDir = path.join(rootDir, "public", "icons");
const sizes = [192, 512];

async function generateIcons() {
  fs.mkdirSync(outputDir, { recursive: true });

  for (const size of sizes) {
    await sharp(sourceIcon)
      .resize(size, size, { fit: "cover" })
      .png()
      .toFile(path.join(outputDir, `icon-${size}.png`));
  }
}

generateIcons().catch((err) => {
  console.error(err);
  process.exit(1);
});
