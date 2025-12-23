const CACHE_NAME = 'rust-chat-v1';
const ASSETS = [
    '/',
    '/index.html',
    '/favicon.svg',
    '/manifest.json',
    '/css/base.css',
    '/css/sidebar.css',
    '/css/chat.css',
    '/css/messages.css',
    '/js/config.js',
    '/js/state.js',
    '/js/dom.js',
    '/js/utils.js',
    '/js/features.js',
    '/js/reactions.js',
    '/js/messages.js',
    '/js/websocket.js',
    '/js/events.js',
    '/js/main.js'
];

self.addEventListener('install', (e) => {
    e.waitUntil(
        caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS))
    );
});

self.addEventListener('fetch', (e) => {
    e.respondWith(
        caches.match(e.request).then((res) => res || fetch(e.request))
    );
});
