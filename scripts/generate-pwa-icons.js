#!/usr/bin/env node
/**
 * 生成 PWA 图标到 apps/mobile/public/icons/ 目录。
 *
 * manifest.json 引用 /icons/icon-192.png 与 /icons/icon-512.png，
 * 由于 manifest 的 base URL 是移动 PWA 的 public 目录，所以图标必须
 * 输出到 apps/mobile/public/icons/ 而非根 public/icons/。
 */
import fs from 'fs';
import path from 'path';
import sharp from 'sharp';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const candidates = [
  path.join(rootDir, 'apps', 'mobile', 'public', 'app-icon.png'),
  path.join(rootDir, 'app-icon.png'),
  path.join(rootDir, 'public', 'app-icon.png'),
];
const sourceIcon = candidates.find((p) => fs.existsSync(p));

if (!sourceIcon) {
  throw new Error(
    'Missing source icon: app-icon.png (looked in apps/mobile/public/, root, public/)'
  );
}

// manifest.json 引用 /icons/icon-*.png，其相对于 apps/mobile/public/ 解析。
const outputDir = path.join(rootDir, 'apps', 'mobile', 'public', 'icons');
const sizes = [192, 512];

async function generateIcons() {
  fs.mkdirSync(outputDir, { recursive: true });

  for (const size of sizes) {
    await sharp(sourceIcon)
      .resize(size, size, { fit: 'cover' })
      .png()
      .toFile(path.join(outputDir, `icon-${size}.png`));
    console.log(`Generated ${path.relative(rootDir, path.join(outputDir, `icon-${size}.png`))}`);
  }
}

generateIcons().catch((err) => {
  console.error(err);
  process.exit(1);
});
