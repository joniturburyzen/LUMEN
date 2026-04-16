// ─── SERVICE WORKER: ACTUALIZACIÓN GARANTIZADA ───────────────────────────────
// v22 — Rutas absolutas para funcionar en subdirectorios + mejor error handling
// ─────────────────────────────────────────────────────────────────────────────

const CACHE = "lumen-v22";

// Obtener la ruta base dinámicamente (funciona en root y subdirectorios)
const BASE_PATH = location.pathname.substring(0, location.pathname.lastIndexOf('/')) || '';

// Archivos mínimos para funcionar offline (rutas absolutas desde la raíz del scope)
const PRECACHE = [
  location.pathname,           // URL actual (normalmente index.html)
  BASE_PATH + '/index.html',
  BASE_PATH + '/manifest.json',
  BASE_PATH + '/icon-192.png',
  BASE_PATH + '/icon-512.png',
  BASE_PATH + '/pkg/lumen_quill.js',
  BASE_PATH + '/pkg/lumen_quill_bg.wasm',
];

// ── INSTALL ───────────────────────────────────────────────────────────────────
self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE).then((cache) => cache.addAll(PRECACHE))
  );
  self.skipWaiting(); // activa inmediatamente
});

// ── ACTIVATE ──────────────────────────────────────────────────────────────────
self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(keys.filter(k => k !== CACHE).map(k => caches.delete(k)))
    )
  );
  self.clients.claim(); // controla todas las pestañas
});

// ── FETCH ─────────────────────────────────────────────────────────────────────
self.addEventListener("fetch", (event) => {
  const req = event.request;
  const url = new URL(req.url);

  // Solo GET
  if (req.method !== "GET") return;

  // 1) HTML SIEMPRE desde la red (network-first)
  if (req.mode === "navigate") {
    event.respondWith(
      fetch(req)
        .then((res) => {
          if (res && res.status === 200) {
            caches.open(CACHE).then((c) => c.put(req, res.clone()));
          }
          return res;
        })
        .catch(() => caches.match(BASE_PATH + '/index.html'))
    );
    return;
  }

  // 2) JS y WASM → network-first (siempre intenta la red, fallback a caché)
  if (url.pathname.endsWith(".js") || url.pathname.endsWith(".wasm")) {
    event.respondWith(
      fetch(req)
        .then((res) => {
          if (res && res.status === 200) {
            caches.open(CACHE).then((c) => c.put(req, res.clone()));
          }
          return res;
        })
        .catch(() => caches.match(req))
    );
    return;
  }

  // 3) Resto de assets → cache-first
  event.respondWith(
    caches.match(req).then((cached) => {
      if (cached) return cached;

      return fetch(req).then((res) => {
        if (!res || res.status !== 200) return res;
        caches.open(CACHE).then((c) => c.put(req, res.clone()));
        return res;
      });
    }).catch(() => null)
  );
});
