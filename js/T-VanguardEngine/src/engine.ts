import {  AdBlockEngine, AdBlockExclusionStore, AdBlockRequest, assemble_resources } from "vanguard";
import { Parser } from "htmlparser2";
import { EngineData } from "./data";
import { inject } from "./rewrite";

export type EngineRequestParams = {
    type: string;
    uuid: string;
    url: string;
    source: string;
    method: string;
};
export type EngineResponseParams = {
    request: EngineRequestParams;
    body: ReadableStream | ArrayBuffer | Blob | string;
    headers: Record<string, string | string[]>;
    status: number;
    statusText: string;
};
export type EngineRequestResult = {
    req?: EngineRequestParams;
    resp?: EngineResponseParams;
    error?: Error;
    handleResponse: boolean;
};
export interface Engine {
    ready: boolean;
    init(fetch: typeof window.fetch): Promise<void>;
    onMessage(message: any): Promise<any>;
    onRequest(request: EngineRequestParams): Promise<EngineRequestResult>;
    onResponse(response: EngineResponseParams): Promise<EngineResponseParams>;
}
export type EngineInit = {
    uuidChannelName: string;
    storeName: string,
    storeVersion: number,
    objectStore: string,
    assetsPath?: string;
}

export class VanguardEngine implements Engine {
    engineInit: EngineInit; // @ts-ignore
    engineData: Awaited<EngineData>; // @ts-ignore
    engine: Awaited<AdBlockEngine>; // @ts-ignore
    exclusions: Awaited<AdBlockExclusionStore>;
    uuidChannel: BroadcastChannel; // @ts-ignore
    timers: {
        saveStats: { lastTimeMs: number, limitMs: number }
    }
    ready: boolean;

    static EngineData = EngineData;

    constructor(init: EngineInit) {
        this.uuidChannel = new BroadcastChannel(init.uuidChannelName); 
        this.engineInit = init;
        this.ready = false;
    }

    async init(fetch: typeof window.fetch) {
        try {
            this.engineData = await EngineData.fromExistingStore(
                fetch,
                this.engineInit.storeName,
                this.engineInit.storeVersion,
                this.engineInit.objectStore
            );
        } catch (_) {
            this.engineData = await EngineData.fromAssetsJson(
                fetch,
                this.engineInit.storeName,
                this.engineInit.storeVersion,
                this.engineInit.objectStore,
                this.engineInit.assetsPath!
            );
            // defer
            (async () => { await this.engineData.save() })();
        }
        this.engine = new AdBlockEngine(await this.engineData.assembleFilterSet());
        this.engine.use_resources(assemble_resources(await this.engineData.assembleResources()).toArray());
        this.exclusions = new AdBlockExclusionStore(this.engineData.userExclusions.split('\n').map(x => x.trim()));
        this.timers = {
            saveStats: { lastTimeMs: performance.now(), limitMs: 5000 }
        }
        this.ready = true;
    }

    tryHandleSaveRequestStats() {
        const saveStats = this.timers.saveStats;
        const now = performance.now()
        if (now - saveStats.lastTimeMs > saveStats.limitMs){
            saveStats.lastTimeMs = now;
            (async () => {
                await EngineData.util.saveAllowedRequests(
                    this.engineData.storeName,
                    this.engineData.storeVersion,
                    this.engineData.objectStore,
                    this.engineData.allowedRequests
                );
                await EngineData.util.saveBlockedRequests(
                    this.engineData.storeName,
                    this.engineData.storeVersion,
                    this.engineData.objectStore,
                    this.engineData.blockedRequests
                );
            })();
        }
    }

    async onMessage(message: any): Promise<any> {
        // reload, get, set
        throw new Error("Method not implemented.");
    }

    async onRequest(request: EngineRequestParams): Promise<EngineRequestResult> {
        const result = await (async () => {
            const url = new URL(request.url);
            const exclusion = this.exclusions.matchHost(url.hostname);
            if (exclusion) {
                this.engineData.allowedRequests++;
                this.uuidChannel.postMessage({
                    type: 'ExclusionEntry',
                    value: {
                        type: request.type,
                        uuid: request.uuid,
                        url: request.url,
                        source: request.source,
                        method: request.method,
                        result: exclusion
                    }
                });
                return { req: request, handleResponse: false };
            }
            
            const abr = new AdBlockRequest(request.url, request.source, request.type);
            const res = this.engine.check_network_request(abr);

            this.uuidChannel.postMessage({
                type: 'BlockerEntry',
                value: {
                    type: request.type,
                    uuid: request.uuid,
                    url: request.url,
                    source: request.source,
                    method: request.method,
                    result: EngineData.util.resultToSendable(res)
                }
            });
            // If it didn't match, return ok. If it did, see if it should be blocked or not
            if (!res.matched) {
                this.engineData.allowedRequests++;
                return { req: request, handleResponse: true };
            }
            // Must block, even if there's an exception
            if (res.important) {
                this.engineData.blockedRequests++;
                let msg = `${request.method} (${request.type}): ${request.url} from ${request.source} (${request.uuid})`;
                return { error: new Error(`Failed to load resource: net::ERR_BLOCKED_BY_CLIENT: ${msg}`), handleResponse: false };
            }
            // There was a match, but exception rule override them
            if (res.exception) {
                this.engineData.allowedRequests++;
                return { req: request, handleResponse: false }
            };
            // response body should be resource
            if (res.redirect) {
                this.engineData.allowedRequests++;
                // redirect is a data url. It won't actually fetch anything
                const resp = await fetch(res.redirect);
                return { resp: { request, body: resp.body || '', headers: Object.fromEntries(resp.headers), status: resp.status, statusText: resp.statusText }, handleResponse: false }
            }
            // Rewritten url to use
            if (res.rewritten_url) {
                this.engineData.allowedRequests++;
                return { req: { ...request, url: res.rewritten_url }, handleResponse: true }
            };
            // Otherwise, block it
            this.engineData.blockedRequests++;
            let msg = `${request.method} (${request.type}): ${request.url} from ${request.source} (${request.uuid})`;
            return { error: new Error(`Failed to load resource: net::ERR_BLOCKED_BY_CLIENT: ${msg}`), handleResponse: false };
        })();
        this.tryHandleSaveRequestStats();
        return result;
    } 
    async onResponse(response: EngineResponseParams): Promise<EngineResponseParams> {
        // Only rewrite html
        const ctypeHeader = response.headers["content-type"] || "";
        const contentType = Array.isArray(ctypeHeader) ? ctypeHeader[0] : (ctypeHeader || '');
        const body = new Response(response.body).body;
        if (!(contentType.startsWith("text/html") && body)) return response;
        const ids: string[] = [];
        const classes: string[] = [];
        const decoder = new TextDecoder();
        const whitespace = /\s+/;
        const url_specific_resources = EngineData.util.urlResourcesToSendable(this.engine.url_cosmetic_resources(response.request.url));
        const parsedInfo = { html : '', bodyCloseStartIndex : -1, htmlCloseStartIndex : -1 };
        const parser = new Parser({
            onattribute: url_specific_resources.generichide ? () => {} : (name, value) => {
                if (name === 'id')
                    ids.push(value);
                else if (name === 'class')
                    value.split(whitespace).forEach(entry => classes.push(entry));
            },
            onclosetag(name) {
                if (name === "body")
                    parsedInfo.bodyCloseStartIndex = parser.startIndex;
                else if (name === "html")
                    parsedInfo.htmlCloseStartIndex = parser.startIndex;
            }
        });

        const reader = body.getReader();
        while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            const decoded = decoder.decode(value);
            parsedInfo.html += decoded;
            parser.write(decoded);
        }
        parser.end();

        if (!url_specific_resources.generichide) {
            url_specific_resources.hide_selectors.push(...this.engine.hidden_class_id_selectors(
                ids,
                classes,
                url_specific_resources.exceptions,
            ));
        }
        
        this.uuidChannel.postMessage({
            type: 'CosmeticsEntry',
            value: {
                type: response.request.type,
                uuid: response.request.uuid,
                url: response.request.url,
                source: response.request.source,
                method: response.request.method,
                result: url_specific_resources
            }
        });

        return { ...response, body: inject(parsedInfo, url_specific_resources) };
    }
}