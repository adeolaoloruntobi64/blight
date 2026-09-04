import { CONFIG } from "../config";

export const TRANSPORT_DEFAULTS = {
    bare: {
        endpoint: CONFIG.bareV3Url,
        version: 3,
    },
    epoxy: {
        wisp_v2: false,
        wisp: CONFIG.wispV1Url,
        wasm: CONFIG.epoxyWasm
    },
    libcurl: {
        websocket: CONFIG.wsproxyUrl,
        wasm: CONFIG.libcurlWasm,
        transport: "wsproxy"
    },
};

export type TransportName = keyof typeof TRANSPORT_DEFAULTS;
export const TRANSPORT_NAMES = Object.keys(TRANSPORT_DEFAULTS) as TransportName[];