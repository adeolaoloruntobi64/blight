import type { IDBStore } from "./idb";
import type { VanguardSyncChannel } from "./sync";

export type Stats = { blocked: number; allowed: number };

export class StatsTracker {
    #pendingBlocked = 0;
    #pendingAllowed = 0;
    #totalBlocked = 0;
    #totalAllowed = 0;
    #flushHandle: number | null = null;
    #listeners = new Set<(stats: Stats) => void>();

    constructor(private readonly store: IDBStore, private readonly sync: VanguardSyncChannel) {
        sync.onStatsUpdated((stats) => {
            this.#totalBlocked = stats.blocked;
            this.#totalAllowed = stats.allowed;
            this.#notify();
        });
    }

    async init(): Promise<void> {
        this.#totalBlocked = (await this.store.get<number>("blockedRequests")) ?? 0;
        this.#totalAllowed = (await this.store.get<number>("allowedRequests")) ?? 0;
        this.#notify();
    }

    get totals(): Stats {
        return { blocked: this.#totalBlocked, allowed: this.#totalAllowed };
    }

    onUpdate(callback: (stats: Stats) => void): void {
        this.#listeners.add(callback);
    }

    #notify(): void {
        for (const cb of this.#listeners)
            cb(this.totals);
    }

    recordBlocked(): void {
        this.#pendingBlocked++;
        this.#totalBlocked++;
        this.#notify();
        this.#scheduleFlush();
    }

    recordAllowed(): void {
        this.#pendingAllowed++;
        this.#totalAllowed++;
        this.#notify();
        this.#scheduleFlush();
    }

    #scheduleFlush(): void {
        if (this.#flushHandle !== null) return;
        this.#flushHandle = window.setTimeout(async () => {
            this.#flushHandle = null;
            const blockedDelta = this.#pendingBlocked;
            const allowedDelta = this.#pendingAllowed;
            this.#pendingBlocked = 0;
            this.#pendingAllowed = 0;
            if (blockedDelta === 0 && allowedDelta === 0)
                return;
            const [newBlocked, newAllowed] = await Promise.all([
                blockedDelta ? this.store.increment("blockedRequests", blockedDelta) : this.#totalBlocked,
                allowedDelta ? this.store.increment("allowedRequests", allowedDelta) : this.#totalAllowed,
            ]);
            this.#totalBlocked = newBlocked;
            this.#totalAllowed = newAllowed;
            this.sync.announceStatsUpdated({ blocked: newBlocked, allowed: newAllowed });
        }, 5000);
    }
}