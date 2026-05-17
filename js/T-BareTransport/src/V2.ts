import { Client, statusEmpty } from "./Client";
import { BareError } from "./BareTypes";
import md5 from "./md5.js";
import { joinHeaders, splitHeaders } from "./splitHeaderUtil.js";
import type {
	ProxyTransport,
	RawHeaders,
	TransferrableResponse,
} from "@mercuryworkshop/proxy-transports";
import { urlToRemote } from "./remoteUtil";

export default class ClientV2 extends Client implements ProxyTransport {
	ws: URL;
	http: URL;
	newMeta: URL;
	getMeta: URL;
    ready: boolean;
    supportsStreaming: boolean;

    meta() {
		return {};
	}

	constructor(server: URL) {
		super(2, server);

		this.ws = new URL(this.base);
		this.http = new URL(this.base);
		this.newMeta = new URL('./ws-new-meta', this.base);
		this.getMeta = new URL(`./ws-meta`, this.base);

		if (this.ws.protocol === 'https:') {
			this.ws.protocol = 'wss:';
		} else {
			this.ws.protocol = 'ws:';
		}

        // @ts-ignore It exists bruv
		switch (performance.getEntries()[0].nextHopProtocol) {
			case "h2":
			case "h2c":
			case "h3":
				this.supportsStreaming = true;
				break;
            case "http/0.9":
            case "http/1.0":
            case "http/1.1":
			default:
				this.supportsStreaming = false;
        }

        this.ready = true;
	}
        
    async init() {}

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
        const barev2ws = (async () => {
            const request = new Request(this.newMeta, {
                headers: this.createBareHeaders(url, {
                    ...requestHeaders,
                    // I'm not sure if these are included
                    // in the headers or not
                    ...{ "Origin": url.origin },
                    ...protocols.length != 0 && { "Sec-WebSocket-Protocol": protocols.join() }
                }),
            });
    
            const assignMeta = await fetch(request);
            if (!assignMeta.ok) throw new BareError(assignMeta.status, await assignMeta.json());
    
            const id = await assignMeta.text();
            const socket: WebSocket = new WebSocket(this.ws, id);

            socket.addEventListener('open', async () => {
                const outgoing = await fetch(this.getMeta, { headers: { 'x-bare-id': id }, method: 'GET' });
                const resp = await this.readBareResponse(outgoing, true);
                onopen(resp.rawHeaders["Sec-WebSocket-Protocol"] || "", "");
            });
            socket.addEventListener("message", (ev) => {
				onmessage(ev.data);
			});
            socket.addEventListener("close", (ev) => {
				onclose(ev.code, ev.reason);
			});
            socket.addEventListener("error", (ev) => {
				onerror(JSON.stringify(ev));
			});
    
            return socket;
        })();

        return [ 
			async (data) => {
				(await barev2ws).send(data);
			},
			async (code, reason) => {
				(await barev2ws).close(code, reason)
			}
		]
    }

    async request(
        remote: URL,
        method: string,
        body: BodyInit | null,
        headers: RawHeaders,
        signal: AbortSignal | undefined
    ): Promise<TransferrableResponse> {
        if (remote.protocol === 'blob:') {
			const response = await fetch(remote);
			return {
                body: response.body!,
                headers: Array.from(response.headers.entries()) as RawHeaders,
                status: response.status,
                statusText: response.statusText
            };
		}

		const forwardHeaders = ['accept-encoding', 'accept-language'];

		const options: RequestInit = {
			credentials: 'omit',
			method: method,
			signal,
			...this.supportsStreaming && { duplex: "half" }
		};

		if (body) {
			if (!this.supportsStreaming && body instanceof ReadableStream) {
				options.body = await new Response(body).blob()
			} else {
				options.body = body;
			}
		}

        options.headers = this.createBareHeaders(remote, headers, forwardHeaders);

		// bare can be an absolute path containing no origin, it becomes relative to the script
		const response = await fetch(
			this.http + '?cache=' + md5(remote.toString()),
			options
		);
		const readResponse = await this.readBareResponse(response);

		return {
            body: statusEmpty.includes(readResponse.status) ? "" : (response.body || ""),
            headers: Array.from(readResponse.headers.entries()) as RawHeaders,
            status: readResponse.status, // NO WAY BRO I FORGOT TO CHANGE THIS TO READRESPONSE IT TOOK ME 1H 30M TO FIND AAAAAAAAAAAAAAAAAA
            statusText: readResponse.statusText // NO WAY BRO I FORGOT TO CHANGE THIS TO READRESPONSE IT TOOK ME 1H 30M TO FIND AAAAAAAAAAAAAAAAAA
        };
    }
    
	private async readBareResponse(
		response: Response,
		webSocket = false
	) {
		if (!response.ok) {
			throw new BareError(response.status, await response.json());
		}

		const responseHeaders = joinHeaders(response.headers);
		const xBareStatus = responseHeaders.get('x-bare-status');
		const xBareStatusText = responseHeaders.get('x-bare-status-text');
		const xBareHeaders = responseHeaders.get('x-bare-headers');
		const rawHeaders = xBareHeaders ? JSON.parse(xBareHeaders) : {}

		return {
            status: xBareStatus !== null ? parseInt(xBareStatus) : webSocket ? 101 : 200,
            statusText: xBareStatusText !== null ? xBareStatusText : webSocket ? 'Switching Protocols' : 'OK',
            headers: new Headers(rawHeaders as HeadersInit),
            rawHeaders
        };
	}

	createBareHeaders(
		remote: URL,
		bareHeaders: RawHeaders,
		forwardHeaders: string[] = [],
		passHeaders: string[] = [],
		passStatus: number[] = []
	) {
		const headers = new Headers();
		const bareRemote = urlToRemote(remote);
		headers.set('x-bare-protocol', bareRemote.protocol);
		headers.set('x-bare-host', bareRemote.host);
		headers.set('x-bare-path', bareRemote.path);
		headers.set('x-bare-port', bareRemote.port.toString());
		headers.set('x-bare-headers', JSON.stringify(bareHeaders));
        headers.set('x-bare-forward-headers', forwardHeaders.join());
		headers.set('x-bare-pass-headers', passHeaders.join());
		headers.set('x-bare-forward-status', passStatus.join());
		return splitHeaders(headers);
	}
}