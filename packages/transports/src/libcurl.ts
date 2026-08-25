import type { RawHeaders, TransferrableResponse, ProxyTransport } from "@mercuryworkshop/proxy-transports";
// libcurl.js does not currently provide TypeScript declarations.
// @ts-expect-error Missing declaration file for the JavaScript package.
import { libcurl } from "libcurl.js";

// slightly modified version of https://github.com/MercuryWorkshop/libcurl-transport/

export type LibcurlTransportOptions = {
    wasm: string;
    websocket: string;
    proxy?: string;
    transport?: string;
    connections?: Array<number>;
};

export default class LibcurlTransport implements ProxyTransport {
    ready = false;
    session: any;
    wasm: string;
    websocket: string;
    proxy?: string;
    transport?: string;
    connections?: Array<number>;

    constructor(options: LibcurlTransportOptions) {
        this.websocket = options.websocket;
        this.wasm = options.wasm;
        this.transport = options.transport;
        this.proxy = options.proxy;
        this.connections = options.connections;
        if (!this.websocket.endsWith("/")) {
            throw new TypeError(
                "The Websocket URL must end with a trailing forward slash."
            );
        }
        if (!this.websocket.startsWith("ws://") && !this.websocket.startsWith("wss://")) {
            throw new TypeError(
                "The Websocket URL must use the ws:// or wss:// protocols."
            );
        }
        if (typeof options.proxy === "string") {
            let protocol = new URL(options.proxy).protocol;
            if (!["socks5h:", "socks4a:", "http:"].includes(protocol)) {
                throw new TypeError(
                    "Only socks5h, socks4a, and http proxies are supported."
                );
            }
        }
    }

    async init() {
        await libcurl.load_wasm(this.wasm);
        if (this.transport)
            libcurl.transport = this.transport;
        if (!libcurl.ready) {
            await new Promise((resolve, reject) => {
                libcurl.onload = () => {
                    console.log("loaded libcurl.js v" + libcurl.version.lib);
                    this.ready = true;
                    resolve(null);
                };
            });
        }
        libcurl.set_websocket(this.websocket);
        this.session = new libcurl.HTTPSession({
            proxy: this.proxy,
        });
        if (this.connections) this.session.set_connections(...this.connections);
        this.ready = libcurl.ready;
        if (this.ready) {
            console.log("running libcurl.js v" + libcurl.version.lib);
            return;
        }
    }

    async meta() { }

    async request(
        remote: URL,
        method: string,
        body: BodyInit | null,
        headers: RawHeaders,
        signal: AbortSignal | undefined
    ): Promise<TransferrableResponse> {
        let headersObj: Record<string, string> = {};
        for (let [key, value] of headers)
            headersObj[key] = value;
        let payload = await this.session.fetch(remote.href, {
            method,
            headers: headersObj,
            body,
            redirect: "manual",
            signal: signal,
        });
        return {
            body: payload.body!,
            headers: payload.raw_headers,
            status: payload.status,
            statusText: payload.statusText,
        };
    }

    connect(
        url: URL,
        protocols: string[],
        requestHeaders: RawHeaders,
        onopen: (protocol: string, extensions: string) => void,
        onmessage: (data: Blob | ArrayBuffer | string) => void,
        onclose: (code: number, reason: string) => void,
        onerror: (error: string) => void
    ): [
            (data: Blob | ArrayBuffer | string) => void,
            (code: number, reason: string) => void,
        ] {
        let headersObj: Record<string, string> = {};
        for (let [key, value] of requestHeaders)
            headersObj[key] = value;
        let socket = new libcurl.WebSocket(url.toString(), protocols, {
            headers: headersObj,
        });
        socket.binaryType = "arraybuffer";
        // @ts-ignore
        socket.onopen = (event: Event) => {
            onopen("", "");
        };
        // @ts-ignore
        socket.onclose = (event: CloseEvent) => {
            onclose(event.code, event.reason);
        };
        // @ts-ignore
        socket.onerror = (event: Event) => {
            onerror("");
        };
        // @ts-ignore
        socket.onmessage = (event: MessageEvent) => {
            onmessage(event.data);
        };
        return [
            (data) => socket.send(data),
            // @ts-ignore
            (code, reason) => socket.close(code, reason),
        ];
    }
}

export { LibcurlTransport };