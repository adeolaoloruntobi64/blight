export abstract class Client {
	protected base: URL;
	/**
	 *
	 * @param version Version provided by extension
	 * @param server Bare Server URL provided by BareClient
	 */
	constructor(version: number, server: URL) {
		this.base = new URL(`./v${version}/`, server);
	}
}

export const statusEmpty = [101, 204, 205, 304];
export const statusRedirect = [301, 302, 303, 307, 308];
