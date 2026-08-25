import ClientV1 from "./bare/V1";
import ClientV2 from "./bare/V2";
import ClientV3 from "./bare/V3";
import type { ProxyTransport, RawHeaders, TransferrableResponse } from "@mercuryworkshop/proxy-transports";

// Inspired by https://github.com/MercuryWorkshop/bare-transport/

export type BareVersion = 1 | 2 | 3;

export type BareOptions = {
    version: BareVersion,
    endpoint: string
};

type InternalClient = ClientV1 | ClientV2 | ClientV3;

export default class BareTransport implements ProxyTransport {
	private client: InternalClient;

	constructor(options: BareOptions) {
		switch (options.version) {
			case 1:
				this.client = new ClientV1(options.endpoint);
				break;
			case 2:
				this.client = new ClientV2(options.endpoint);
				break;
			case 3:
				this.client = new ClientV3(options.endpoint);
				break;
			default: 
				throw new Error(`Unsupported Bare version: ${options.version}`);
		}
	}

    get ready() {
        return this.client.ready;
    }

    set ready(value) {
        this.client.ready = value;
    }

	async init() {
		await this.client.init();
	}

	connect(
		url: URL,
		protocols: string[],
		requestHeaders: RawHeaders,
		onopen: (protocol: string, extensions: string) => void,
		onmessage: (data: Blob | ArrayBuffer | string) => void,
		onclose: (code: number, reason: string) => void,
		onerror: (error: string) => void
	) {
		return this.client.connect(url, protocols, requestHeaders, onopen, onmessage, onclose, onerror);
	}

	async request(
		remote: URL,
		method: string,
		body: BodyInit | null,
		headers: RawHeaders,
		signal: AbortSignal | undefined
	): Promise<TransferrableResponse> {
		return this.client.request(remote, method, body, headers, signal);
	}
}