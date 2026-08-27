import { CONFIG } from "./config";

declare const self: ServiceWorkerGlobalScope;

importScripts(CONFIG.scramjetControlerSw);

self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) =>
    event.waitUntil(caches.keys().then((keys) => Promise.all(
        keys.filter(
            (key) =>
                key.startsWith(CONFIG.shellCachePrefix) &&
                key !== CONFIG.shellCache
        )
            .map((key) => caches.delete(key))
    )).then(() => self.clients.claim()))
);

self.addEventListener("fetch", (event) => {
    if (self.$scramjetController?.shouldRoute(event)) 
        return event.respondWith(self.$scramjetController.route(event));
    if (isShellRequest(new URL(event.request.url)))
        event.respondWith(shellResponse(event.request));
});

const isUnderRoot = (pathname: string, root: string) =>
    pathname === root.slice(0, -1) || pathname.startsWith(root);

const isShellRequest = (url: URL) => !(
    url.pathname.startsWith("/@vite")
    || url.pathname.startsWith("/@fs")
    || url.pathname.startsWith("/@id")
    || url.pathname.includes(".vite/")
    || /\.(tsx|ts|jsx|css)$/.test(url.pathname)
    || url.search
    || ['localhost', '127.0.0.1', '::1'].includes(location.hostname)
    || url.pathname.startsWith(CONFIG.swScope)
    || CONFIG.runtimeRoots.some((root) => isUnderRoot(url.pathname, root))
);

const shellResponse = async (request: Request) => {
    const cache = await caches.open(CONFIG.shellCache);
    const cached = await cache.match(request);
    const network = fetch(request)
        .then((response) => {
            if (response.ok) cache.put(request, response.clone());
            return response;
        })
        .catch(() => cached ?? Response.error());
    return cached ?? network;
};