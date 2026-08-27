import { TRANSPORT_DEFAULTS, type TransportName } from "./defaults";
import type { IDBStore } from "../vanguard/idb";

export async function getTransportOptions<T>(
    store: IDBStore, name: TransportName
): Promise<T> {
    const stored = await store.get<T>(`${name}-options`);
    return stored ?? TRANSPORT_DEFAULTS[name] as T;
}

export async function setTransportOptions<T>(
    store: IDBStore, name: TransportName, options: T
): Promise<void> {
    await store.put(`${name}-options`, options);
}

export async function resetTransportOptions(store: IDBStore, name: TransportName): Promise<void> {
    await store.put(`${name}-options`, TRANSPORT_DEFAULTS[name]);
}

export async function resetAllTransportOptions(store: IDBStore): Promise<void> {
    for (const name of Object.keys(TRANSPORT_DEFAULTS) as TransportName[]) {
        await store.put(`${name}-options`, TRANSPORT_DEFAULTS[name]);
    }
}