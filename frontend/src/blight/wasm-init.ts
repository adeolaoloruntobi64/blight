import { ContentBlockingConversionResult } from "vanguard";
import { CONFIG } from "./config";
import { IDBStore } from "./vanguard/idb";

// Todo: Complete this
let createVanguardPromise: Promise<void> | null = null;
let createBareTransportPromise: Promise<void> | null = null;
let createEpoxyTransportPromise: Promise<void> | null = null;
let createLibCurlTransportPromise: Promise<void> | null = null;
let createScramjetControllerPromise: Promise<void> | null = null;

async function createVanguard() {
    let { default: wasmInit, ContentBlockingConversionResult } = await import(CONFIG.vanguard) as typeof import("vanguard");
    await wasmInit({ module_or_path: CONFIG.vanguardWasm });
}

async function createBareTransport() {
    
}

async function createEpoxyTransport() {
    const { default: EpoxyTransport } = await import("@mercuryworkshop/epoxy-transport");
    //const b  = new EpoxyTransport({
    //    wisp_v2?: boolean;
    //    udp_extension_required?: boolean;
    //    title_case_headers?: boolean;
    //    ws_title_case_headers?: boolean;
    //    wisp_ws_protocols?: string[];
    //    redirect_limit?: number;
    //    header_limit?: number;
    //    buffer_size?: number;
    //});
}

async function createLibCurlTransport() {

}

async function createTransport() {
    const a = await import(CONFIG.epoxy);
    // 
    
}

export function getVanguardPromise() {
    if (!createVanguardPromise)
        createVanguardPromise = createVanguard();
    return createVanguardPromise;
}