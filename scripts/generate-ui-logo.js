#!/usr/bin/env node
/**
 * 生成共享包品牌 logo 到 packages/ui/src/assets/logo.png（§2）。
 *
 * 从根 app-icon.png（2048×2048）resize 到 256×256（方→方等价恒等缩放，不裁剪区域）。
 * logo 随共享包走、源码模块 import：各端 Vite emit 哈希资产，不寄生任何 public/。
 * 幂等：重复执行产出同尺寸文件。
 */
import fs from 'fs';
import path from 'path';
import sharp from 'sharp';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

const candidates = [
  path.join(rootDir, 'app-icon.png'),
  path.join(rootDir, 'apps', 'mobile', 'public', 'app-icon.png'),
  path.join(rootDir, 'public', 'app-icon.png'),
];
const sourceIcon = candidates.find((p) => fs.existsSync(p));

if (!sourceIcon) {
  throw new Error(
    'Missing source icon: app-icon.png (looked in root, apps/mobile/public/, public/)'
  );
}

const outputFile = path.join(rootDir, 'packages', 'ui', 'src', 'assets', 'logo.png');
const size = 256;

async function generateLogo() {
  fs.mkdirSync(path.dirname(outputFile), { recursive: true });
  await sharp(sourceIcon).resize(size, size, { fit: 'cover' }).png().toFile(outputFile);
  console.log(`Generated ${path.relative(rootDir, outputFile)} (${size}×${size})`);
}

generateLogo().catch((err) => {
  console.error(err);
  process.exit(1);
});
