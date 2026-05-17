import { RawHeaders } from '@mercuryworkshop/proxy-transports';
import type { BareRemote } from './remoteUtil';

export interface BareV1Meta {
	remote: BareRemote;
	headers: RawHeaders;
	forward_headers: string[];
	id?: string;
}

export interface BareV1MetaRes {
	headers: RawHeaders;
}