export const fetch = globalThis.fetch;
export const WebSocket = globalThis.WebSocket;
export const Request = globalThis.Request;
export const Response = globalThis.Response;
export const XMLHttpRequest = globalThis.XMLHttpRequest;

export const WebSocketFields = {
	prototype: {
		send: WebSocket.prototype.send,
	},
	CLOSED: WebSocket.CLOSED,
	CLOSING: WebSocket.CLOSING,
	CONNECTING: WebSocket.CONNECTING,
	OPEN: WebSocket.OPEN,
};
