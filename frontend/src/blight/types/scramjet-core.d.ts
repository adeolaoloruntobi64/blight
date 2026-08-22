import type { ScramjetClient as ScramjetClientBase } from "./scramjet-hooks";

export interface ScramjetHeaders {
    set(key: string, value: string): void;
    get(key: string): string | null;
    has(key: string): boolean;
    delete(key: string): void;
    clone(): ScramjetHeaders;
    toRawHeaders(): [string, string][];
    toNativeHeaders(): Headers;
}

export interface CookieJar {
    setCookies(cookieString: string, url: URL): void;
    getCookies(url: URL, fromJs: boolean, sameSiteContext?: "strict" | "lax" | "cross-site"): string;
    load(cookies: string): void;
    dump(): string;
    clear(): void;
}