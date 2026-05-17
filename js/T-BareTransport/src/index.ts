import { TransferrableResponse, ProxyTransport, RawHeaders } from "@mercuryworkshop/proxy-transports";
import ClientV1 from "./V1";
import ClientV2 from "./V2";
import ClientV3 from "./V3";

export default class BareTransport implements ProxyTransport {
	client: ClientV1 | ClientV2 | ClientV3

    meta() {
		return this.client.meta();
	}

    get ready() {
        return this.client.ready;
    }

    set ready(ready: boolean) {
        this.client.ready = ready;
    }

	constructor(endpoint: URL, version: "v1" | "v2" | "v3") {
		switch (version) {
            case "v1":
                this.client = new ClientV1(endpoint);
                break;
            case "v2":
                this.client = new ClientV2(endpoint);
                break;
            case "v3":
                this.client = new ClientV3(endpoint);
                break;
            default:
                throw `Invalid version ${version}. Must be 'v1', 'v2', or 'v3'`
        }
	}
        
    async init() {
        await this.client.init();
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
       return this.client.connect(url, protocols, requestHeaders, onopen, onmessage, onclose, onerror);
    }

    async request(
        remote: URL,
        method: string,
        body: BodyInit | null,
        headers: RawHeaders,
        signal: AbortSignal | undefined
    ): Promise<TransferrableResponse> {
        return await this.client.request(remote, method, body, headers, signal);
    }
}