import { BareError } from "./BareTypes";
import type { BareResponseHeaders, SocketClientToServer, SocketServerToClient } from "./V3Types";
import { md5 } from "js-md5";
import { WebSocketFields } from "./snapshot";
import { joinHeaders, splitHeaders } from "./splitHeaderUtil";
import type { ProxyTransport, RawHeaders, TransferrableResponse } from "@mercuryworkshop/proxy-transports";

export default class ClientV3 implements ProxyTransport {
	ws: URL;
	http: URL;
	ready = true;

	constructor(endpoint: string) {
		this.ws = new URL(endpoint);
		this.http = new URL(endpoint);
		if (this.ws.protocol === "https:")
			this.ws.protocol = "wss:";
		else
			this.ws.protocol = "ws:";
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
		const ws = new WebSocket(this.ws);
		requestHeaders.push(["Host", url.host])
		requestHeaders.push(["Upgrade", "websocket"])
		requestHeaders.push(["Connection", "Upgrade"])
		const cleanup = () => {
			ws.removeEventListener("close", closeListener);
			ws.removeEventListener("message", messageListener);
			ws.removeEventListener("error", errorListener);
		};
		const messageListener = (event: MessageEvent) => {
			cleanup();
			if (typeof event.data !== "string") {
				onerror("the first websocket message was not a text frame");
				return;
			}
			const message = JSON.parse(event.data) as SocketServerToClient;
			if (message.type !== "open") {
				onerror("message was not of open type");
				return;
			}
			onopen(message.protocol, "");
			ws.addEventListener("message", (ev) => {
				onmessage(ev.data);
			});
			ws.addEventListener("close", (ev) => {
				onclose(ev.code, ev.reason);
			});
		};
		const closeListener = (event: CloseEvent) => {
			onclose(event.code, event.reason);
			cleanup();
		};
		const errorListener = (event: Event) => {
			onerror("Websocket err");
		}
		ws.addEventListener("message", messageListener);
		ws.addEventListener("close", closeListener);
		ws.addEventListener("error", errorListener);
		ws.addEventListener(
			"open",
			(event) => {
				WebSocketFields.prototype.send.call(
					ws,
					JSON.stringify({
						type: "connect",
						remote: url.toString(),
						protocols,
						headers: Object.fromEntries(requestHeaders),
						forwardHeaders: [],
					} as unknown as SocketClientToServer)
				);
			},
			{ once: true }
		);
		return [ws.send.bind(ws), ws.close.bind(ws)];
	}

	async request(
		remote: URL,
		method: string,
		body: BodyInit | null,
		headers: RawHeaders,
		signal: AbortSignal | undefined
	): Promise<TransferrableResponse> {
		const options: RequestInit = {
			credentials: "omit",
			method: method,
			signal,
			//@ts-expect-error not typed on RequestInit but supported at runtime
			duplex: "half",
		};
		if (body !== undefined)
			options.body = body;
		headers.push(["Host", remote.host])
		options.headers = this.createBareHeaders(remote, headers);
		const response = await fetch(
			this.http + "?cache=" + md5(remote.toString()),
			options
		);
		const readResponse = await this.readBareResponse(response);
		return {
			body: response.body!,
			headers: readResponse.headers,
			status: readResponse.status,
			statusText: readResponse.statusText,
		};
	}

	private async readBareResponse(response: Response) {
		if (!response.ok)
			throw new BareError(response.status, await response.json());
		const responseHeaders = joinHeaders(response.headers);
		const result: Partial<BareResponseHeaders> = {};
		const xBareStatus = responseHeaders.get("x-bare-status");
		if (xBareStatus !== null) result.status = parseInt(xBareStatus);
		const xBareStatusText = responseHeaders.get("x-bare-status-text");
		if (xBareStatusText !== null) result.statusText = xBareStatusText;
		const xBareHeaders = responseHeaders.get("x-bare-headers");
		if (xBareHeaders !== null) result.headers = Object.entries(JSON.parse(xBareHeaders));
		return result as BareResponseHeaders;
	}
	createBareHeaders(
		remote: URL,
		bareHeaders: RawHeaders,
		forwardHeaders: string[] = [],
		passHeaders: string[] = [],
		passStatus: number[] = []
	) {
		const headers = new Headers();
		headers.set("x-bare-url", remote.toString());
		headers.set("x-bare-headers", JSON.stringify(Object.fromEntries(bareHeaders)));
		for (const header of forwardHeaders)
			headers.append("x-bare-forward-headers", header);
		for (const header of passHeaders)
			headers.append("x-bare-pass-headers", header);
		for (const status of passStatus)
			headers.append("x-bare-pass-status", status.toString());
		splitHeaders(headers);
		return headers;
	}
}
