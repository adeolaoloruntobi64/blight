// https://developer.mozilla.org/en-US/docs/Web/API/ReadableStreamDefaultReader/read
// https://stackoverflow.com/questions/61296252/failed-to-execute-put-on-idbobjectstore-the-transaction-has-finished
// Always compress data before accessing the db or else the transaction will cut and fail

import { openDB } from "idb";
import { AdBlockBlockerResult, AdBlockUrlSpecificResources, AdBlockFilterListMetadata, AdBlockExpiresIntervalType } from "vanguard";

export type SendableBlockerResult = {
    exception?: string;
    filter?: string;
    important: boolean;
    matched: boolean;
    redirect?: string;
    rewritten_url?: string;
}

export type SendableUrlSpecificResources = {
    exceptions: string[],
    generichide: boolean,
    hide_selectors: string[],
    procedural_actions: string[],
    injected_script: string,
};

export enum SendableExpiresIntervalType {
    Hours = 0,
    Days = 1
};

export type SendableFilterListMetadata = {
    expires?: { interval_type: SendableExpiresIntervalType, amount: number };
    homepage?: string;
    redirect?: string;
    title?: string;
  }

export type AdBlockSendEntry = {
    type: string,
    url: string,
    bresult?: SendableBlockerResult,
    ureselt?: SendableUrlSpecificResources
}

export type IDBItem<T> = {
    key: string,
    value: T
}

const util = {

    /* ====================================================================================== */
    /*                        Converters to Safe Sendable Objects                             */
    /* ====================================================================================== */

    resultToSendable: (a: AdBlockBlockerResult): SendableBlockerResult => {
        return {
            matched: a.matched,
            important: a.important,
            ...a.filter && {filter: a.filter},
            ...a.redirect && {redirect: a.redirect},
            ...a.exception && {exception: a.exception},
            ...a.rewritten_url && {rewritten_url: a.rewritten_url}
        };
    },
    
    urlResourcesToSendable: (u: AdBlockUrlSpecificResources): SendableUrlSpecificResources => {
        return {
            exceptions: u.exceptions,
            generichide: u.generichide,
            hide_selectors: u.hide_selectors,
            procedural_actions: u.procedural_actions,
            injected_script: u.injected_script,
        }
    },

    filterListMetadataToSendable: (l: AdBlockFilterListMetadata): SendableFilterListMetadata => {
        return {
            ...l.expires && {
                expires: {
                    interval_type:
                        (l.expires.interval_type === AdBlockExpiresIntervalType.Days) ?
                        SendableExpiresIntervalType.Days : SendableExpiresIntervalType.Hours,
                    amount: l.expires.amount
                }
            },
            ...l.homepage && { homepage: l.homepage },
            ...l.redirect && { redirect: l.redirect },
            ...l.title && { title: l.title },
        }
    },

    /* ====================================================================================== */
    /*                             Compression and Decompression                              */
    /* ====================================================================================== */

    compressBlob: async (blob: Blob, algo: CompressionFormat = 'gzip') => {
        const stream = blob.stream();
        const compressedStream = stream.pipeThrough<Uint8Array>(new CompressionStream(algo));
        const chunks: Uint8Array[] = [];
        const reader = compressedStream.getReader();
        while (true) {
            const { done, value } = await reader.read();
            if (done) return await util.concatUint8Arrays(chunks);
            chunks.push(value);
        }
    },

    decompressBlob: async (blob: Blob, algo: CompressionFormat = 'gzip') => {
        const stream = blob.stream();
        const decompressedStream = stream.pipeThrough(new DecompressionStream(algo));
        const chunks: Uint8Array[] = [];
        const reader = decompressedStream.getReader();
        while (true) {
            const { done, value } = await reader.read();
            if (done) return await util.concatUint8Arrays(chunks);
            chunks.push(value);
        }
    },

    concatUint8Arrays: async (uint8arrays: Uint8Array[]): Promise<Uint8Array> => {
        return new Uint8Array(await new Blob(uint8arrays as BlobPart[]).arrayBuffer());
    },

    /* ====================================================================================== */
    /*                                IndexedDb Utility Functions                             */
    /* ====================================================================================== */

    openIDB: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
    ) => {
        return await openDB(storeName, storeVersion, {
            upgrade: (db) => {
                db.createObjectStore(objectStore, { keyPath: "key" });
            }
        });
    },

    // Reads Can Fail
    getStoreRead: async(
        storeName: string,
        storeVersion: number,
        objectStore: string,
    ) => {
        const idb = await util.openIDB(storeName, storeVersion, objectStore);
        const tx = idb.transaction(objectStore, 'readonly');
        return tx.objectStore(objectStore);
    },

    // Writes are never expected to fail
    getStoreWrite: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
    ) => {
        const idb = await util.openIDB(storeName, storeVersion, objectStore);
        if (!idb.objectStoreNames.contains(objectStore))
            idb.createObjectStore(objectStore);
        const tx = idb.transaction(objectStore, 'readwrite');
        return tx.objectStore(objectStore);
    },

    /* ====================================================================================== */
    /*                          Non-Compressing Loaders and Savers                            */
    /* ====================================================================================== */

    loadAvailableFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ) => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = await store.get('availableFilters');
        return entry.value;
    },

    saveAvailableFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        availableFilters: Object
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = { key: 'availableFilters', value: availableFilters };
        await store.put(entry);
    },

    loadImportedLists: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<string[]> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<string[]> = await store.get('importedLists');
        return entry.value;
    },

    saveImportedLists: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        importedLists: string[]
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<string[]> = { key: 'importedLists', value: importedLists };
        await store.put(entry);
    },

    loadSelectedFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<string[]> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<string[]> = await store.get('selectedFilters');
        return entry.value;
    },

    saveSelectedFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        selectedFilters: string[]
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<string[]> = { key: 'selectedFilters', value: selectedFilters };
        await store.put(entry);
    },

    loadUserFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<string> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<string> = await store.get('userFilters');
        return entry.value;
    },

    saveUserFilters: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        userFilters: string
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<string> = { key: 'userFilters', value: userFilters };
        await store.put(entry);
    },

    loadUserExclusions: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<string> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<string> = await store.get('userExclusions');
        return entry.value;
    },

    saveUserExclusions: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        userExclusions: string
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<string> = { key: 'userExclusions', value: userExclusions };
        await store.put(entry);
    },

    loadBlockedRequests: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<number> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<number> = await store.get('blockedRequests');
        return entry.value;
    },

    saveBlockedRequests: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        blockedRequests: number
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<number> = { key: 'blockedRequests', value: blockedRequests };
        await store.put(entry);
    },

    loadAllowedRequests: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<number> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<number> = await store.get('allowedRequests');
        return entry.value;
    },

    saveAllowedRequests: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        allowedRequests: number
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<number> = { key: 'allowedRequests', value: allowedRequests };
        await store.put(entry);
    },

    loadAssetsJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<Object> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = await store.get('assetsJsonEntry');
        return entry.value;
    },

    saveAssetsJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        assetsJsonEntry: Object
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = { key: 'assetsJsonEntry', value: assetsJsonEntry };
        await store.put(entry);
    },

    loadAssembleJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<Object> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = await store.get('assembleJsonEntry');
        return entry.value;
    },

    saveAssembleJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        assembleJsonEntry: Object
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = { key: 'assembleJsonEntry', value: assembleJsonEntry };
        await store.put(entry);
    },

    loadBadlistsJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<Object> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = await store.get('badlistsJsonEntry');
        return entry.value;
    },

    saveBadlistsJsonEntry: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        badlistsJsonEntry: Object
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = { key: 'badlistsJsonEntry', value: badlistsJsonEntry };
        await store.put(entry);
    },

    loadAssembleJson: async (
        storeName: string,
        storeVersion: number,
        objectStore: string
    ): Promise<Object> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = await store.get('assembleJson');
        return entry.value;
    },

    saveAssembleJson: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        assembleJson: Object
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Object> = { key: 'assembleJson', value: assembleJson };
        await store.put(entry);
    },

    /* ====================================================================================== */
    /*                              Compressing Loaders and Savers                            */
    /* ====================================================================================== */

    loadFilterList: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        filterListName: string
    ): Promise<string> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Uint8Array> = await store.get(`cache/list/${filterListName}`);
        const decompressed = await util.decompressBlob(new Blob([entry.value as BlobPart]));
        const decoder = new TextDecoder();
        return decoder.decode(decompressed);
    },

    saveFilterList: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        filterListName: string,
        filterListContents: string
    ) => {
        const compressed = await util.compressBlob(new Blob([filterListContents]));
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Uint8Array> = { key: `cache/list/${filterListName}`, value: compressed };
        await store.put(entry);
    },

    loadResource: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        resourceName: string
    ): Promise<Uint8Array> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<Uint8Array> = await store.get(`cache/resource/${resourceName}`);
        return await util.decompressBlob(new Blob([entry.value as BlobPart]));
    },

    saveResource: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        resourceName: string,
        resourceContents: Uint8Array
    ) => {
        const compressed = await util.compressBlob(new Blob([resourceContents as BlobPart]));
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<Uint8Array> = { key: `cache/resource/${resourceName}`, value: compressed };
        await store.put(entry);
    },

    /* ====================================================================================== */
    /*                         Filter List Meta Loaders and Savers                            */
    /* ====================================================================================== */

    loadFilterListMetadata: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        filterListName: string
    ): Promise<SendableFilterListMetadata> => {
        const store = await util.getStoreRead(storeName, storeVersion, objectStore);
        const entry: IDBItem<SendableFilterListMetadata> = await store.get(`cache/meta/${filterListName}`);
        return entry.value;
    },

    saveFilterListMetadata: async (
        storeName: string,
        storeVersion: number,
        objectStore: string,
        filterListName: string,
        meta: SendableFilterListMetadata
    ) => {
        const store = await util.getStoreWrite(storeName, storeVersion, objectStore);
        const entry: IDBItem<SendableFilterListMetadata> = { key: `cache/meta/${filterListName}`, value: meta };
        await store.put(entry);
    },
};

export default util;