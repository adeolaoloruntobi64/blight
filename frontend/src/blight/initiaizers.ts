import { CONFIG } from "./config";
import { IDBStore } from "./vanguard/idb";
import type { ProxyTransport } from "@mercuryworkshop/proxy-transports";
import type { BareOptions } from "transports/bare";
import type { EpoxyOptions } from "transports/epoxy";
import type { LibcurlTransportOptions } from "transports/libcurl";
import { VanguardSyncChannel } from "./vanguard/sync";
import { VanguardStore } from "./vanguard/vstore";
import { StatsTracker } from "./vanguard/stats";
import { VanguardHandle } from "./vanguard/handle";
import { getTransportOptions, setTransportOptions } from "./transports/options";
import { TRANSPORT_DEFAULTS } from "./transports/defaults";

// To get around vite complaining about importing from public/
const import2 = new Function('path', 'return import(path)');

export async function createServiceWorker() {
    const registration = await navigator.serviceWorker.register(CONFIG.sw, { scope: CONFIG.swScope });
    await navigator.serviceWorker.ready;
    if (!navigator.serviceWorker.controller) {
        await new Promise<void>((resolve) => {
            const onChange = () => {
                if (navigator.serviceWorker.controller) {
                    navigator.serviceWorker.removeEventListener("controllerchange", onChange);
                    resolve();
                }
            };
            navigator.serviceWorker.addEventListener("controllerchange", onChange);
        });
    }
    const worker = navigator.serviceWorker.controller ?? registration.active;
    if (!worker) throw new Error("No active service worker after registration");
    return worker;
}

export async function createVanguard(store: IDBStore, transport: ProxyTransport) {
    const {
        default: wasmInit, VanguardEngine, VanguardExclusionStore, VanguardRequest,
        VanguardFilterSet, VanguardParseOptions, VanguardResourceAssemblerInfo,
        VanguardInlineWebAcessibleResources, VanguardAssembledResources, assemble_resources
    } = await import2(CONFIG.vanguard) as typeof import("vanguard");
    const types = {
        VanguardFilterSet,
        VanguardParseOptions,
        VanguardResourceAssemblerInfo,
        VanguardInlineWebAcessibleResources,
        VanguardAssembledResources,
        assemble_resources,
    };
    await wasmInit({ module_or_path: CONFIG.vanguardWasm });
    const buildEngineFromData = async (data: VanguardStore) => {
        const filterSet = await data.assembleFilterSet();
        // Note to self: If you move engine initialization ONE LINE DOWN, the app doesn't work
        // Why? https://www.rossng.eu/posts/2025-01-20-wasm-bindgen-pitfalls/. Incredible.
        // Do NOT hold wasm objects over an await. This was an incredible headache.
        const engine = new VanguardEngine(filterSet);
        const resources = await data.assembleResources();
        engine.use_resources(resources.toArray());
        const exclude = new VanguardExclusionStore(data.userExclusions.split("\n"));
        return { engine, exclude };
    };
    const pxfetch = async (input: URL | RequestInfo, init?: RequestInit) => {
        const req = new Request(input, init);
        const resp = await transport.request(
            new URL(req.url),
            req.method,
            req.body,
            req.headers as any,
            req.signal
        );
        const rinit = {
            headers: resp.headers,
            status: resp.status,
            statusText: resp.statusText,
        };
        return new Response(resp.body, rinit);
    };
    const sync = new VanguardSyncChannel();
    const vstore = await VanguardStore.init(types, fetch, pxfetch, store, CONFIG.assetsJson);
    const stats = new StatsTracker(store, sync);
    const { engine, exclude } = await buildEngineFromData(vstore);
    const holder = new VanguardHandle(engine, exclude);
    sync.onConfigChanged(async () => {
        await vstore.loadAll();
        const rebuilt = await buildEngineFromData(vstore);
        holder.replaceEngine(rebuilt.engine);
        holder.replaceExclude(rebuilt.exclude);
    });
    const updateFilterLists = async (newLists: string[]) => {
        vstore.selectedFilters = newLists;
        await vstore.save("selectedFilters");
        sync.announceConfigChanged();
        const rebuilt = await buildEngineFromData(vstore);
        holder.replaceEngine(rebuilt.engine);
        holder.replaceExclude(rebuilt.exclude);
    }
    return { vstore, holder, stats, sync, updateFilterLists, VanguardRequest }
}

export async function createScramjetController(worker: ServiceWorker, transport: ProxyTransport) {
    return new globalThis.$scramjetController.Controller({
        serviceworker: worker,
        transport,
        config: {
            prefix: CONFIG.prefix,
            scramjetPath: CONFIG.scramjet,
            wasmPath: CONFIG.scramjetWasm,
            injectPath: CONFIG.scramjetControllerInject,
            codec: {
                encode: (url) => {
                    if (!url) return url;
                    return encodeURIComponent(url); // btoa for base64
                },
                decode: (url) => {
                    if (!url) return url;
                    return decodeURIComponent(url); // atob for base64
                }
            }
        },
    });
}

async function createBareTransport(store: IDBStore) {
    const { default: BareTransport } = await import2(CONFIG.bare) as typeof import ("transports/bare");
    let options = await getTransportOptions<BareOptions>(store, "bare");
    if (!options) {
        options = TRANSPORT_DEFAULTS.bare as any;
        await setTransportOptions<BareOptions>(store, "bare", options);
    }
    return new BareTransport(options);
}

async function createEpoxyTransport(store: IDBStore) {
    const { default: EpoxyTransport } = await import2(CONFIG.epoxy) as typeof import("transports/epoxy");
    let options = await getTransportOptions<EpoxyOptions>(store, "epoxy");
    if (!options) {
        options = TRANSPORT_DEFAULTS.epoxy as any;
        await setTransportOptions<EpoxyOptions>(store, "epoxy", options);
    }
    return new EpoxyTransport(options);
}

async function createLibcurlTransport(store: IDBStore) {
    const { default: LibcurlTransport } = await import2(CONFIG.libcurl) as typeof import("transports/libcurl");
    let options = await getTransportOptions<LibcurlTransportOptions>(store, "libcurl");
    if (!options) {
        options = TRANSPORT_DEFAULTS.libcurl as any;
        await setTransportOptions<LibcurlTransportOptions>(store, "libcurl", options);
    }
    return new LibcurlTransport(options);
}

export async function createTransport(store: IDBStore) {
    const transportName = await store.get<string>("active-transport-name");
    if (!transportName) {
        await store.put("active-transport-name", "epoxy");
        return await createEpoxyTransport(store);
    }
    if (transportName == "bare")
        return await createBareTransport(store);
    if (transportName == "epoxy")
        return await createEpoxyTransport(store);
    if (transportName == "libcurl")
        return await createLibcurlTransport(store);
    throw Error(transportName + " is not a valid transport");
}