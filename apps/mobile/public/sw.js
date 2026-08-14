// DropVoice Mobile Service Worker
// 仅在 HTTPS / localhost 环境下注册（见 src/main.tsx）。
// 策略：网络优先，失败时回退到缓存，保证 PWA 离线可用基础壳。

const CACHE_NAME = 'dropvoice-mobile-v1';
const PRECACHE_URLS = ['/', '/index.html', '/manifest.json'];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      .then((cache) => cache.addAll(PRECACHE_URLS))
      .catch(() => undefined)
  );
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key)))
      )
      .then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  const { request } = event;
  if (request.method !== 'GET') return;

  // §9.8：WebRTC 架构无 WebSocket，删除旧的 websocket 守卫。

  event.respondWith(
    fetch(request)
      .then((response) => {
        // 仅缓存同源的成功响应。
        if (response.ok && new URL(request.url).origin === self.location.origin) {
          const clone = response.clone();
          caches
            .open(CACHE_NAME)
            .then((cache) => cache.put(request, clone))
            .catch(() => undefined);
        }
        return response;
      })
      .catch(() => caches.match(request).then((cached) => cached || Response.error()))
  );
});
