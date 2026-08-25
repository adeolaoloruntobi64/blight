import { IDBStore } from "./idb";
import { compressBlob, decompressBlob } from "./compress";
import { filterListMetadataToSendable, type SendableFilterListMetadata } from "./convert";
import type {
  VanguardFilterSet as VanguardFilterSetClass,
  VanguardParseOptions as VanguardParseOptionsClass,
  VanguardResourceAssemblerInfo as VanguardResourceAssemblerInfoClass,
  VanguardInlineWebAcessibleResources as VanguardInlineWebAcessibleResourcesClass,
  VanguardAssembledResources as VanguardAssembledResourcesClass,
  assemble_resources as assemble_resources_function
} from "vanguard";

export interface VanguardTypes {
  VanguardFilterSet: typeof VanguardFilterSetClass;
  VanguardParseOptions: typeof VanguardParseOptionsClass;
  VanguardResourceAssemblerInfo: typeof VanguardResourceAssemblerInfoClass;
  VanguardInlineWebAcessibleResources: typeof VanguardInlineWebAcessibleResourcesClass;
  VanguardAssembledResources: typeof VanguardAssembledResourcesClass;
  assemble_resources: typeof assemble_resources_function;
}

const SIMPLE_KEYS = [
    "availableFilters", "importedLists", "userExclusions", "selectedFilters",
    "userFilters", "blockedRequests", "allowedRequests", "assetsJsonEntry",
    "assembleJsonEntry", "badlistsJsonEntry", "assembleJson",
] as const;

type SimpleKey = (typeof SIMPLE_KEYS)[number];

const DEFAULT_SELECTED_FILTERS = [
    "ublock-filters", "ublock-badware", "ublock-privacy", "ublock-quick-fixes", "ublock-unbreak",
    "easylist", "adguard-generic", "adguard-mobile", "easyprivacy", "urlhaus-1", "plowe-0", "ublock-annoyances",
];

export class VanguardStore {
    private readonly store: IDBStore;
    private readonly fetch: typeof window.fetch;
    private readonly pxfetch: typeof window.fetch;

    types: VanguardTypes;
    availableFilters: Record<string, any> = {};
    importedLists: string[] = [];
    userExclusions = "";
    selectedFilters: string[] = [];
    userFilters = "";
    blockedRequests = 0;
    allowedRequests = 0;
    assetsJsonEntry: Record<string, any> = {};
    assembleJsonEntry: Record<string, any> = {};
    badlistsJsonEntry: Record<string, any> = {};
    assembleJson: Record<string, any> = {};

    constructor(
        types: VanguardTypes,
        fetch: typeof window.fetch,
        pxfetch: typeof window.fetch,
        store: IDBStore
    ) {
        this.types = types;
        this.fetch = (input, url) => fetch(input, url);
        this.pxfetch = (input, url) => pxfetch(input, url);
        this.store = store;
    }

    getStore(): IDBStore { return this.store; }

    static async init(
        types: VanguardTypes,
        fetch: typeof window.fetch,
        pxfetch: typeof window.fetch,
        store: IDBStore,
        assetsPath: string
    ): Promise<VanguardStore> {
        const data = new VanguardStore(types, fetch, pxfetch, store);

        if (await data.getFlag("__initialized")) {
            await data.loadAll();
            return data;
        }

        await data.#bootstrapFromAssetsJson(assetsPath);
        await data.saveAll();
        await data.setFlag("__initialized", true);
        return data;
    }

    async #bootstrapFromAssetsJson(assetsPath: string): Promise<void> {
        const assets = await (await this.fetch(assetsPath)).json();
        this.assetsJsonEntry = assets["assets.json"];
        this.assembleJsonEntry = assets["assembler.json"];
        this.badlistsJsonEntry = assets["ublock-badlists"];

        delete assets["assets.json"];
        delete assets["assembler.json"];
        delete assets["ublock-badlists"];

        this.assembleJson = await (await this.fetch(this.assembleJsonEntry.contentURL)).json();
        this.availableFilters = assets;
        this.selectedFilters = [...DEFAULT_SELECTED_FILTERS];
        this.importedLists = [];
        this.userFilters = "";
        this.userExclusions = "";
        this.blockedRequests = 0;
        this.allowedRequests = 0;
    }

    async getFlag(key: string): Promise<boolean> {
        return (await this.store.get<boolean>(key)) ?? false;
    }
    async setFlag(key: string, value: boolean): Promise<void> {
        await this.store.put(key, value);
    }

    async saveAll(): Promise<void> {
        const entries: Record<string, unknown> = {};
        for (const key of SIMPLE_KEYS) entries[key] = this[key];
        await this.store.putMany(entries);
    }

    async loadAll(): Promise<void> {
        const values = await this.store.getMany(SIMPLE_KEYS as unknown as string[]);
        for (const key of SIMPLE_KEYS)
            if (values[key] !== undefined) 
                (this as any)[key] = values[key];
    }

    async save<K extends SimpleKey>(key: K): Promise<void> {
        await this.store.put(key, this[key]);
    }
    async load<K extends SimpleKey>(key: K): Promise<void> {
        const value = await this.store.get<VanguardStore[K]>(key);
        if (value !== undefined)
            (this[key] as VanguardStore[K]) = value;
    }

    async increment(key: string, delta: number): Promise<number> {
        return this.store.increment(key, delta);
    }
    async getCounter(key: string): Promise<number> {
        return (await this.store.get<number>(key)) ?? 0;
    }

    async saveFilterList(name: string, contents: string): Promise<void> {
        await this.store.put(`cache/list/${name}`, await compressBlob(new Blob([contents])));
    }
    async loadFilterList(name: string): Promise<string> {
        const compressed = await this.store.get<Uint8Array>(`cache/list/${name}`);
        if (!compressed) throw new Error(`No cached filter list: ${name}`);
        return new TextDecoder().decode(await decompressBlob(new Blob([compressed as BlobPart])));
    }

    async saveResource(name: string, contents: Uint8Array): Promise<void> {
        await this.store.put(`cache/resource/${name}`, await compressBlob(new Blob([contents as BlobPart])));
    }
    async loadResource(name: string): Promise<Uint8Array> {
        const compressed = await this.store.get<Uint8Array>(`cache/resource/${name}`);
        if (!compressed) throw new Error(`No cached resource: ${name}`);
        return decompressBlob(new Blob([compressed as BlobPart]));
    }

    async saveFilterListMetadata(name: string, meta: SendableFilterListMetadata): Promise<void> {
        await this.store.put(`cache/meta/${name}`, meta);
    }

    async assembleFilterSet(): Promise<VanguardFilterSetClass> {
        const withContents = await Promise.all(
            this.selectedFilters.map(async (name) => {
                let contents: string;
                try {
                    contents = await this.loadFilterList(name);
                } catch {
                    // contentUrl is either a string or array of strings. We want the first element
                    // of the array if it is an array, or just the string itself if it is a string.
                    const filterUrl = [this.availableFilters[name].contentURL].flat()[0];
                    // we need to fetch with the transport, else we get cors errors
                    contents = await (await this.pxfetch(filterUrl)).text();
                    void this.saveFilterList(name, contents); // deferred
                }
                return { name, contents };
            })
        );
        return this.assembleFilterSetWithContents(withContents);
    }

    assembleFilterSetWithContents(filterListsWithContents: { name: string; contents: string }[]): VanguardFilterSetClass {
        const filterset = new this.types.VanguardFilterSet(true);
        for (const { name, contents } of filterListsWithContents) {
            const meta = filterset.add_filter_list(contents, this.types.VanguardParseOptions.default());
            void this.saveFilterListMetadata(name, filterListMetadataToSendable(meta)); // deferred
        }
        return filterset;
    }

    async collectResources(): Promise<VanguardResourceAssemblerInfoClass> {
        const rrPath = this.assembleJson["redirect-resources.js"];
        const ssPath = this.assembleJson["scriptlets.js"];
        const webARPaths = Object.entries<string>(this.assembleJson["web_accessible_resources"]);
        const decoder = new TextDecoder();

        const rr = await this.#loadOrFetchResource("redirect-resources.js", rrPath, decoder);
        const ss = await this.#loadOrFetchResource("scriptlets.js", ssPath, decoder);

        const webAR = Object.fromEntries(
            await Promise.all(
                webARPaths.map(async ([name, path]) => {
                    let contents: Uint8Array;
                    try {
                        contents = await this.loadResource(name);
                    } catch {
                        contents = new Uint8Array(await (await this.fetch(path)).arrayBuffer());
                        void this.saveResource(name, contents);
                    }
                    return [name, contents] as const;
                })
            )
        );

        return new this.types.VanguardResourceAssemblerInfo(rr, new this.types.VanguardInlineWebAcessibleResources(webAR), ss);
    }

    async assembleResources(): Promise<VanguardAssembledResourcesClass> {
        return this.types.assemble_resources(await this.collectResources());
    }

    async #loadOrFetchResource(cacheName: string, path: string, decoder: TextDecoder): Promise<string> {
        try {
            return decoder.decode(await this.loadResource(cacheName));
        } catch {
            const bytes = new Uint8Array(await (await this.fetch(path)).arrayBuffer());
            void this.saveResource(cacheName, bytes);
            return decoder.decode(bytes);
        }
    }
}