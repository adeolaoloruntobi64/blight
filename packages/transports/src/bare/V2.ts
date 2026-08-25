import { BareError, BareErrorBody } from "./BareTypes";
import { urlToRemote } from "./remoteUtil";
import { rawHeadersToObject } from "./encodeProtocol";
import { md5 } from "js-md5";
import type { ProxyTransport, RawHeaders, TransferrableResponse } from "@mercuryworkshop/proxy-transports";

export default class ClientV2 implements ProxyTransport {
	http: URL;
	ws: URL;
	newMeta: URL;
	getMeta: URL;
	ready = true;

    constructor(endpoint: string) {
		this.http = new URL(endpoint);
		this.ws = new URL(endpoint);
		this.newMeta = new URL("ws-new-meta", endpoint);
		this.getMeta = new URL("ws-meta", endpoint);
		this.ws.protocol = this.ws.protocol === "https:" ? "wss:" : "ws:";
	}

	async init() {
		this.ready = true;
	}

    connect(
        url: URL,
        protocols: string[],
        requestHeaders: RawHeaders = [],
        onopen: (protocol: string, extensions: string) => void,
        onmessage: (data: Blob | ArrayBuffer | string) => void,
        onclose: (code: number, reason: string) => void,
        onerror: (error: string) => void
    ): [
        (data: Blob | ArrayBuffer | string) => void,
        (code: number, reason: string) => void,
    ] {
        const bareHeaders = rawHeadersToObject(requestHeaders);
        if (protocols.length > 0)
            bareHeaders["Sec-WebSocket-Protocol"] = protocols.join(", ");
        const wsPromise: Promise<WebSocket> = (async () => {
            const assignMeta = await fetch(this.newMeta, {
                headers: this.createBareHeaders(url, bareHeaders),
            });
            if (!assignMeta.ok) {
                const body = (await assignMeta.json()) as BareErrorBody;
                throw new BareError(assignMeta.status, body);
            }
            const id = await assignMeta.text();
            const ws = new WebSocket(this.ws, [id]);
            ws.binaryType = "arraybuffer";
            await new Promise<void>((resolve, reject) => {
                ws.addEventListener("open", () => resolve(), { once: true });
                ws.addEventListener("error", () => reject(new Error("WebSocket error")), { once: true });
            });
            const outgoing = await fetch(this.getMeta, {
                headers: { "x-bare-id": id },
                method: "GET",
            });
            if (!outgoing.ok) {
                const body = (await outgoing.json()) as BareErrorBody;
                ws.close();
                throw new BareError(outgoing.status, body);
            }
            const rawXBareHeaders = outgoing.headers.get("x-bare-headers");
            const metaHeaders: Record<string, string | string[]> = rawXBareHeaders
                ? JSON.parse(rawXBareHeaders)
                : {};
            const negotiatedProtocol = metaHeaders["Sec-WebSocket-Protocol"];
            const protocol = Array.isArray(negotiatedProtocol) ? negotiatedProtocol[0] : negotiatedProtocol ?? "";
            onopen(protocol, "");
            ws.addEventListener("message", (ev) => onmessage(ev.data));
            ws.addEventListener("close", (ev) => onclose(ev.code, ev.reason));
            return ws;
        })();
        wsPromise.catch((e) => onerror(e instanceof Error ? e.message : String(e)));
        return [
            (data) => wsPromise.then((ws) => ws.send(data)),
            (code, reason) => wsPromise.then((ws) => ws.close(code, reason)),
        ];
    }
	async request(
		remote: URL,
		method: string,
		body: BodyInit | null,
		headers: RawHeaders,
		signal: AbortSignal | undefined
	): Promise<TransferrableResponse> {
		const bareHeaders = rawHeadersToObject(headers);
		const options: RequestInit = {
			credentials: "omit",
			method,
			signal,
			// @ts-expect-error not typed on RequestInit but supported at runtime
			duplex: "half",
		};
		if (body !== undefined && body !== null) options.body = body;
		options.headers = this.createBareHeaders(remote, bareHeaders);
		const request = new Request(
			this.http + "?cache=" + md5(remote.toString()),
			options
		);
		const response = await fetch(request);
		if (!response.ok) {
			const errBody = (await response.json()) as BareErrorBody;
			throw new BareError(response.status, errBody);
		}
		const status = response.headers.has("x-bare-status")
			? parseInt(response.headers.get("x-bare-status")!, 10)
			: 200;
		const statusText = response.headers.get("x-bare-status-text") ?? "OK";
		const rawHeaders: RawHeaders = response.headers.has("x-bare-headers")
			? Object.entries(JSON.parse(response.headers.get("x-bare-headers")!))
			: [];
		return {
			body: response.body!,
			headers: rawHeaders,
			status,
			statusText,
		};
	}

	private createBareHeaders(remote: URL, bareHeaders: Record<string, string>) {
		const bareRemote = urlToRemote(remote);
		const headers = new Headers();
		headers.set("x-bare-protocol", bareRemote.protocol);
		headers.set("x-bare-host", bareRemote.host);
		headers.set("x-bare-path", bareRemote.path);
		headers.set("x-bare-port", bareRemote.port.toString());
		headers.set("x-bare-headers", JSON.stringify(bareHeaders));
		return headers;
	}
}