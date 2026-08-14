/**
 * CSP 占位符注入脚本（webrtc-scan-direct-design §5.3）。
 *
 * Tauri 2 的 CSP 是 build-time 静态字段，无原生 env 模板。
 * 本脚本在 tauri build 前根据 NODE_ENV 替换 tauri.conf.json 中的
 * __CSP_CONNECT_SRC__ 占位符：
 *   - production → https://dropvoice.bytehome.fun（信令服务器同域）
 *   - development → http://localhost:38424（本地 pairing-server，跨端口）
 *
 * 用法：在 tauri.conf.json 的 beforeBuildCommand 中调用 `node scripts/inject-csp.mjs`。
 */
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const confPath = join(__dirname, '..', 'tauri.conf.json');

const isProd = process.env.NODE_ENV === 'production';
const cspSrc = isProd
  ? 'https://dropvoice.bytehome.fun'
  : 'http://localhost:38424';

const conf = readFileSync(confPath, 'utf8');
if (!conf.includes('__CSP_CONNECT_SRC__')) {
  console.log('[inject-csp] placeholder already replaced or absent, skipping');
  process.exit(0);
}

const updated = conf.replaceAll('__CSP_CONNECT_SRC__', cspSrc);
writeFileSync(confPath, updated);
console.log(`[inject-csp] injected connect-src: ${cspSrc} (${isProd ? 'production' : 'development'})`);
