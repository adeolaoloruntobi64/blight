import { VanguardStore } from "./vanguard/vstore";
import { VanguardHandle } from "./vanguard/handle";
import { StatsTracker } from "./vanguard/stats";
import { VanguardSyncChannel } from "./vanguard/sync";
import { VanguardEngine, VanguardExclusionStore } from "vanguard";
import type { ControllerLike } from "./types/scramjet-hooks";
import EpoxyTransport from "@mercuryworkshop/epoxy-transport";
import { CONFIG } from "./config";
import { RawHeaders } from "@mercuryworkshop/proxy-transports";

export interface BlightContext {
    controller: ControllerLike;
    vstore: VanguardStore;
    holder: VanguardHandle;
    stats: StatsTracker;
    sync: VanguardSyncChannel;
    updateFilterLists(newLists: string[]): Promise<void>;
}

let bootPromise: Promise<BlightContext> | null = null;

async function buildEngineFromData(data: VanguardStore) {
    const filterSet = await data.assembleFilterSet();
    const resources = await data.assembleResources();
    const engine = new VanguardEngine(filterSet);
    engine.use_resources(resources.toArray());
    const exclude = new VanguardExclusionStore(data.userExclusions.split("\n"));
    return { engine, exclude };
}

export function bootBlight(): Promise<BlightContext> {
    if (!bootPromise)
        bootPromise = doBoot();
    return bootPromise;
}

async function doBoot(): Promise<BlightContext> {
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

    const wispUrl = new URL("/wisp/v1/", location.href);
    wispUrl.protocol = wispUrl.protocol === "https:" ? "wss:" : "ws:";

    const transport = new EpoxyTransport({ wisp_v2: false, udp_extension_required: false, wisp: wispUrl.toString() });
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
    }
    const controller = new globalThis.$scramjetController.Controller({
        serviceworker: worker,
        transport,
        config: {
            prefix: CONFIG.prefix,
            scramjetPath: CONFIG.scramjet,
            wasmPath: CONFIG.scramjetWasm,
            injectPath: CONFIG.scramjetControllerInject,
        },
    });
    await Promise.all([transport.init(), controller.wait()]);

    const sync = new VanguardSyncChannel();
    const vstore = await VanguardStore.init(fetch, pxfetch, "__vanguard", 1, "items", CONFIG.assetsJson);
    const stats = new StatsTracker(vstore.getStore(), sync);
    await stats.init();

    const { engine, exclude } = await buildEngineFromData(vstore);
    const holder = new VanguardHandle(engine, exclude);

    // Wanted to see if I could make the sync channel receive it's own annoucement
    // but a channel instance does not receive it's own post by design. So a small
    // bit of code duplication is needed.
    sync.onConfigChanged(async () => {
        await vstore.loadAll();
        const rebuilt = await buildEngineFromData(vstore);
        holder.replaceEngine(rebuilt.engine);
        holder.replaceExclude(rebuilt.exclude);
    });

    async function updateFilterLists(newLists: string[]) {
        vstore.selectedFilters = newLists;
        await vstore.save("selectedFilters");
        sync.announceConfigChanged();
        const rebuilt = await buildEngineFromData(vstore);
        holder.replaceEngine(rebuilt.engine);
        holder.replaceExclude(rebuilt.exclude);
    }

    return { controller, vstore, holder, stats, sync, updateFilterLists };
}