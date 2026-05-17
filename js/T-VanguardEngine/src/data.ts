import { AdBlockFilterSet, AdBlockParseOptions, AdBlockResourceAssemblerInfo, AdBlockInlineWebAcessibleResources } from "vanguard";
import util from "./util";

export class EngineData {
    storeName: string;
    storeVersion: number;
    objectStore: string;
    availableFilters: Record<string, any>;
    importedLists: string[];
    selectedFilters: string[];      
    userFilters: string;
    userExclusions: string; 
    blockedRequests: number;
    allowedRequests: number;
    assetsJsonEntry: Record<string, any>;
    assembleJsonEntry: Record<string, any>;
    badlistsJsonEntry: Record<string, any>;
    assembleJson: Record<string, any>;
    fetch: typeof window.fetch;

    static util = util;

    constructor(
        storeName: string,
        storeVersion: number,
        objectStore: string,
        availableFilters: Object,
        importedLists: string[],
        selectedFilters: string[],
        userFilters: string,
        userExclusions: string,
        blockedRequests: number,
        allowedRequests: number,
        assetsJsonEntry: Object,
        assembleJsonEntry: Object,
        badlistsJsonEntry: Object,
        assembleJson: Object,
        fetch: typeof window.fetch
    ) {
        this.storeName = storeName;
        this.storeVersion = storeVersion;
        this.objectStore = objectStore;
        this.availableFilters = availableFilters;
        this.importedLists = importedLists;
        this.selectedFilters = selectedFilters;
        this.userFilters = userFilters;
        this.userExclusions = userExclusions;
        this.blockedRequests = blockedRequests;
        this.allowedRequests = allowedRequests;
        this.assetsJsonEntry = assetsJsonEntry;
        this.assembleJsonEntry = assembleJsonEntry;
        this.badlistsJsonEntry = badlistsJsonEntry;
        this.assembleJson = assembleJson;
        this.fetch = fetch;
    }

    static async fromExistingStore(fetch: typeof window.fetch, storeName: string, storeVersion: number, objectStore: string) {
        const availableFilters = await EngineData.util.loadAvailableFilters(storeName, storeVersion, objectStore);
        const importedLists = await EngineData.util.loadImportedLists(storeName, storeVersion, objectStore);
        const selectedFilters = await EngineData.util.loadSelectedFilters(storeName, storeVersion, objectStore);
        const userFilters = await EngineData.util.loadUserFilters(storeName, storeVersion, objectStore);
        const userExclusions = await EngineData.util.loadUserExclusions(storeName, storeVersion, objectStore);
        const blockedRequests = await EngineData.util.loadBlockedRequests(storeName, storeVersion, objectStore);
        const allowedRequests = await EngineData.util.loadAllowedRequests(storeName, storeVersion, objectStore);
        const assetsJsonEntry = await EngineData.util.loadAssetsJsonEntry(storeName, storeVersion, objectStore);
        const assembleJsonEntry = await EngineData.util.loadAssembleJsonEntry(storeName, storeVersion, objectStore);
        const badlistsJsonEntry = await EngineData.util.loadBadlistsJsonEntry(storeName, storeVersion, objectStore);
        const assembleJson = await EngineData.util.loadAssembleJson(storeName, storeVersion, objectStore);
        return new EngineData(
            storeName,
            storeVersion,
            objectStore,
            availableFilters,
            importedLists,
            selectedFilters,
            userFilters,
            userExclusions,
            blockedRequests,
            allowedRequests,
            assetsJsonEntry,
            assembleJsonEntry,
            badlistsJsonEntry,
            assembleJson,
            fetch
        );
    }

    static async fromAssetsJson(fetch: typeof window.fetch, storeName: string, storeVersion: number, objectStore: string, assetsPath: string) {
        const assets = await (await fetch(assetsPath)).json();
        const assetsJsonEntry = assets['assets.json'];
        const assembleJsonEntry = assets['assembler.json'];
        const badlistsJsonEntry = assets['ublock-badlists'];
        
        delete assets['assets.json'];
        delete assets['assembler.json'];
        delete assets['ublock-badlists'];

        const assembleJson = await (await fetch(assembleJsonEntry.contentURL)).json();
        const availableFilters = assets;
        // const selectedFilters: string[] = [];
        // The default filters (not using now because isn't extensive enough)
        // Object.entries<any>(availableFilters).forEach(([key, value]) => {
        //     if (value["group"] === "default")
        //         selectedFilters.push(key);
        // });

        return new EngineData(
            storeName,
            storeVersion,
            objectStore,
            availableFilters,
            [],
            ["ublock-filters", "ublock-badware", "ublock-privacy", "ublock-quick-fixes", "ublock-unbreak", "easylist", "adguard-generic", "adguard-mobile", "easyprivacy", "urlhaus-1", "plowe-0", "ublock-annoyances"],
            "",
            "",
            0,
            0,
            assetsJsonEntry,
            assembleJsonEntry,
            badlistsJsonEntry,
            assembleJson,
            fetch
        );
    }

    async save() {
        await EngineData.util.saveAvailableFilters(this.storeName, this.storeVersion, this.objectStore, this.availableFilters);
        await EngineData.util.saveImportedLists(this.storeName, this.storeVersion, this.objectStore, this.importedLists);
        await EngineData.util.saveUserExclusions(this.storeName, this.storeVersion, this.objectStore, this.userExclusions);
        await EngineData.util.saveSelectedFilters(this.storeName, this.storeVersion, this.objectStore, this.selectedFilters);
        await EngineData.util.saveUserFilters(this.storeName, this.storeVersion, this.objectStore, this.userFilters);
        await EngineData.util.saveBlockedRequests(this.storeName, this.storeVersion, this.objectStore, this.blockedRequests);
        await EngineData.util.saveAllowedRequests(this.storeName, this.storeVersion, this.objectStore, this.allowedRequests);
        await EngineData.util.saveAssetsJsonEntry(this.storeName, this.storeVersion, this.objectStore, this.assetsJsonEntry);
        await EngineData.util.saveAssembleJsonEntry(this.storeName, this.storeVersion, this.objectStore, this.assembleJsonEntry);
        await EngineData.util.saveBadlistsJsonEntry(this.storeName, this.storeVersion, this.objectStore, this.badlistsJsonEntry);
        await EngineData.util.saveAssembleJson(this.storeName, this.storeVersion, this.objectStore, this.assembleJson);
    }

    async assembleFilterSet(): Promise<AdBlockFilterSet> {
        return this.assembleFilterSetWithContents(await Promise.all(
            this.selectedFilters.map(async (name) => {
                let contents: string;
                try {
                    contents = await EngineData.util.loadFilterList(
                        this.storeName,
                        this.storeVersion,
                        this.objectStore,
                        name
                    );
                } catch (_) {
                    contents = await (await this.fetch(this.availableFilters[name].contentURL)).text();
                    // Defer
                    (async () => {
                        await EngineData.util.saveFilterList(
                            this.storeName,
                            this.storeVersion,
                            this.objectStore,
                            name,
                            contents
                        );
                    })();
                }
                return {name, contents};
            })
        ));
    }

    assembleFilterSetWithContents(
        filterListsWithContents: { name: string, contents: string }[]
    ): AdBlockFilterSet {
        const filterset = new AdBlockFilterSet(true);
        filterListsWithContents.forEach(({name, contents}) => {
            const meta = filterset.add_filter_list(contents, AdBlockParseOptions.default());
            // Defer
            (async () => {
                await EngineData.util.saveFilterListMetadata(
                    this.storeName,
                    this.storeVersion,
                    this.objectStore,
                    name,
                    EngineData.util.filterListMetadataToSendable(meta)
                )
            })();
        });
        return filterset;
    }

    async assembleResources(): Promise<AdBlockResourceAssemblerInfo> {
        const rrPath = this.assembleJson['redirect-resources.js'];
        const ssPath = this.assembleJson['scriptlets.js'];
        const webARPaths = Object.entries<string>(this.assembleJson['web_accessible_resources']);

        let rr: string;
        let ss: string | undefined;
        const decoder = new TextDecoder();

        try {
            rr = decoder.decode(await EngineData.util.loadResource(
                this.storeName,
                this.storeVersion,
                this.objectStore,
                'redirect-resources.js'
            ));
        } catch (_) {
            const bytes = new Uint8Array(await (await this.fetch(rrPath)).arrayBuffer());
            rr = decoder.decode(bytes);
            // Defer
            (async () => {
                await EngineData.util.saveResource(
                    this.storeName,
                    this.storeVersion,
                    this.objectStore,
                    'redirect-resources.js',
                    bytes
                )
            })();
        }

        try {
            ss = decoder.decode(await EngineData.util.loadResource(
                this.storeName,
                this.storeVersion,
                this.objectStore,
                'scriptlets.js'
            ));
        } catch (_) {
            if (ssPath) {
                const bytes = new Uint8Array(await (await this.fetch(ssPath)).arrayBuffer());
                ss = decoder.decode(bytes);
                // Defer
                (async () => {
                    await EngineData.util.saveResource(
                        this.storeName,
                        this.storeVersion,
                        this.objectStore,
                        'scriptlets.js',
                        bytes
                    )
                })();
            }
        }

        const webAR = Object.fromEntries(await Promise.all(
            webARPaths.map(async ([name, path]) => {
                let contents: Uint8Array;
                try {
                    contents = await EngineData.util.loadResource(
                        this.storeName,
                        this.storeVersion,
                        this.objectStore,
                        name
                    );
                } catch (_) {
                    contents = new Uint8Array(await (await this.fetch(path)).arrayBuffer());
                    // Defer
                    (async () => {
                        await EngineData.util.saveResource(
                            this.storeName,
                            this.storeVersion,
                            this.objectStore,
                            name,
                            contents
                        )
                    })();
                }
                return [name, contents];
            })
        ));

        const inlineWebAR = new AdBlockInlineWebAcessibleResources(webAR);
        return new AdBlockResourceAssemblerInfo(rr, inlineWebAR, ss);
    }
}
