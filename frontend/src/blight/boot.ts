import type { VanguardRequest as VanguardRequestClass } from "vanguard";
import { VanguardStore } from "./vanguard/vstore";
import { VanguardHandle } from "./vanguard/handle";
import { StatsTracker } from "./vanguard/stats";
import { VanguardSyncChannel } from "./vanguard/sync";
import type { ControllerLike } from "./types/scramjet-hooks";
import { IDBStore } from "./vanguard/idb";
import { createScramjetController, createServiceWorker, createTransport, createVanguard } from "./initiaizers";

export interface BlightContext {
    controller: ControllerLike;
    vstore: VanguardStore;
    holder: VanguardHandle;
    stats: StatsTracker;
    sync: VanguardSyncChannel;
    updateFilterLists(newLists: string[]): Promise<void>;
    VanguardRequest: typeof VanguardRequestClass;
}

let bootPromise: Promise<BlightContext> | null = null;

export function bootBlight(): Promise<BlightContext> {
    if (!bootPromise)
        bootPromise = doBoot();
    return bootPromise;
}

async function doBoot(): Promise<BlightContext> {
    const blightidb = new IDBStore("__blight", 1, "items");
    const [worker, transport] = await Promise.all([
        createServiceWorker(),
        createTransport(blightidb)
    ]);
    await transport.init();
    const [vgbundle, controller] = await Promise.all([
        createVanguard(blightidb, transport),
        createScramjetController(worker, transport)
    ]);
    await Promise.all([controller.wait(), vgbundle.stats.init()]);
    return { controller, ...vgbundle };
}