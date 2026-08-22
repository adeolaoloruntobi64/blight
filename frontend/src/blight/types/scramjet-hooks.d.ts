import type { DomHandler } from "domhandler";
import type { BareCompatibleClient, BareRequestInit, BareResponse, RawHeaders, ProxyTransport } from "@mercuryworkshop/proxy-transports";

export interface Hook<Context, Props> {
    readonly __context?: Context;
    readonly __props?: Props;
}

export interface TapOrder {
    before?: readonly string[];
    after?: readonly string[];
}

export type BodyType = string | ArrayBuffer | Blob | ReadableStream<any>;

export interface ScramjetHeaders {
    set(key: string, value: string): void;
    get(key: string): string | null;
    has(key: string): boolean;
    delete(key: string): void;
    clone(): ScramjetHeaders;
    toRawHeaders(): RawHeaders;
    toNativeHeaders(): Headers;
}

export interface URLMeta {
    origin: URL;
    base: URL;
    topFrameName?: string;
    parentFrameName?: string;
    referrerPolicy?: string;
}

export interface ScramjetFetchTrackedClient {
    [key: string]: unknown;
}

export interface ScramjetFetchRequest {
    rawUrl: URL;
    rawReferrer: string | null;
    rawDestination: RequestDestination;
    mode: RequestMode;
    referrer: string;
    method: string;
    body: BodyType | null;
    cache: RequestCache;
    initialHeaders: ScramjetHeaders;
    rawClientUrl?: URL;
    clientId: string;
}

export interface ScramjetFetchParsed {
    url: URL;
    clientUrl?: URL;
    referrerSourceUrl?: URL | null;
    hadExtraParams: boolean;
    crossSiteRedirect: boolean;
    fetchSiteState?: "same-origin" | "same-site" | "cross-site";
    fetchInitiatorOrigin?: string;
    fetchCredentialsInclude?: boolean;
    fetchMode?: RequestMode;
    isIframe?: boolean;
    destination: RequestDestination;
    meta: URLMeta;
    isModule: boolean;
    isFakeDataURL: boolean;
    referrerPolicy?: string;
    trackedClient?: ScramjetFetchTrackedClient;
}

export interface ScramjetFetchResponse {
    body: BodyType;
    headers: ScramjetHeaders;
    status: number;
    statusText: string;
}

export interface FetchInterceptContext { request: ScramjetFetchRequest; parsed: ScramjetFetchParsed }
export interface FetchInterceptProps { response?: ScramjetFetchResponse }

export interface FetchRequestContext {
    request: ScramjetFetchRequest;
    parsed: ScramjetFetchParsed;
    client: BareCompatibleClient;
}
export interface FetchRequestProps {
    init: BareRequestInit;
    url: URL;
    earlyResponse?: BareResponse | Response;
}

export interface FetchPreresponseContext { request: ScramjetFetchRequest; parsed: ScramjetFetchParsed }
export interface FetchPreresponseProps { response: BareResponse }

export interface FetchResponseContext { request: ScramjetFetchRequest; parsed: ScramjetFetchParsed }
export interface FetchResponseProps { response: ScramjetFetchResponse }

export interface FetchHooks {
    intercept: Hook<FetchInterceptContext, FetchInterceptProps>;
    request: Hook<FetchRequestContext, FetchRequestProps>;
    preresponse: Hook<FetchPreresponseContext, FetchPreresponseProps>;
    response: Hook<FetchResponseContext, FetchResponseProps>;
}

export interface HtmlContext { [key: string]: unknown }
export interface HtmlRewriteContext {
    handler: DomHandler;
    origHtml: string;
    meta: URLMeta;
    htmlcontext: HtmlContext;
}
export interface HtmlRewriteProps { setRawHtml?: string }

export interface HtmlRewriterHooks {
    pre: Hook<HtmlRewriteContext, HtmlRewriteProps>;
    post: Hook<HtmlRewriteContext, HtmlRewriteProps>;
}

export interface NavigateContext { type: "location" | "history" | "hashchange" }
export interface NavigateProps { url: string }

export interface LifecycleHooks {
    navigate: Hook<NavigateContext, NavigateProps>;
}

export interface ScramjetClient {
    url: string;
    global: Window & typeof globalThis;
    meta: URLMeta;
    context: unknown;
    initHeaders: ScramjetHeaders;
    history: unknown;
    bare: BareCompatibleClient;
    serviceWorker: unknown;
    natives: unknown;
    descriptors: unknown;
    rewriteUrl(url: string | URL, options?: unknown): string;
    unrewriteUrl(url: string | URL): string;
    flagEnabled(flag: string): boolean;
    hooks: {
        lifecycle: LifecycleHooks;
        rewriter: { html: HtmlRewriterHooks };
    };
}

export interface InitContext {
    window: Window & typeof globalThis;
    client: ScramjetClient;
    isTopLevel: boolean;
}
export type InitProps = Record<string, never>;

export interface RawHeaders extends Array<[string, string]> { }

export interface TransferRequest {
    rawUrl: string;
    rawReferrer: string | null;
    referrer: string;
    destination: RequestDestination;
    mode: RequestMode;
    method: string;
    body: BodyType | null;
    cache: RequestCache;
    initialHeaders: RawHeaders;
    forceCrossOriginIsolated: boolean;
    rawClientUrl?: string;
    clientId?: string;
}

export interface TransferResponse {
    body: BodyType;
    status: number;
    statusText: string;
    headers: RawHeaders;
}

export interface ErrorRequestContext { rawrequest: TransferRequest; error: unknown }
export interface ErrorRequestProps { setResponse?: TransferResponse; suppressError?: boolean }

export interface FrameInitHooks {
    pre: Hook<InitContext, InitProps>;
    post: Hook<InitContext, InitProps>;
}
export interface FrameErrorHooks {
    request: Hook<ErrorRequestContext, ErrorRequestProps>;
}

export interface FrameHooks {
    fetch: FetchHooks;
    init: FrameInitHooks;
    error: FrameErrorHooks;
}

export interface FetchHandlerLike {
    client: BareCompatibleClient;
    hooks: {
        fetch: FetchHooks;
        rewriter: { html: HtmlRewriterHooks };
    };
}

export interface ScramjetContext {
    config: unknown;
    prefix: URL;
    cookieJar: CookieJar;
    interface: unknown;
}

export interface FrameLike {
    id: string;
    prefix: string;
    element: HTMLIFrameElement;
    controller: ControllerLike;
    plugins: ManagedPluginBaseLike[];
    options: CreateFrameOptions;
    hooks: FrameHooks;
    fetchHandler: FetchHandlerLike;
    readonly context: ScramjetContext;
    go(url: string): void;
    back(): void;
    forward(): void;
    reload(): void;
    getPlugin(name: string): ManagedPluginBaseLike;
}

export declare abstract class PluginBaseLike {
    constructor(name: string, tapOrder?: TapOrder);
    name: string;
    tap<C, P>(
        hook: Hook<C, P>,
        callback: (context: C, props: P) => void | Promise<void>,
        order?: TapOrder
    ): void;
}

export declare abstract class ManagedPluginBaseLike extends PluginBaseLike {
    constructor(name: string, dependencies: string[]);
    dependencies: string[];
    frame?: FrameLike;
    install(frame: FrameLike): void;
}

export interface ControllerConfig {
    prefix: string;
    scramjetPath: string;
    wasmPath: string;
    injectPath: string;
}

export interface ControllerOptions {
    serviceworker: ServiceWorker;
    transport: ProxyTransport;
    config?: Partial<ControllerConfig>;
    scramjetConfig?: Partial<ScramjetConfigLike>;
}

export interface CreateFrameOptions {
    plugins: ManagedPluginBaseLike[];
}

export declare class ControllerLike {
    readonly id: string;
    readonly prefix: string;
    readonly config: unknown;
    readonly scramjetConfig: unknown;
    readonly transport: ProxyTransport;
    readonly cookieJar: CookieJar;
    readonly frames: FrameLike[];
    readonly serviceWorkerController: ServiceWorker;

    constructor(options: ControllerOptions);
    wait(): Promise<void>;
    createFrame(element?: HTMLIFrameElement, options?: CreateFrameOptions): FrameLike;
    setTransport(transport: ProxyTransport): void;
    persistCookies(): Promise<void>;
    propagateCookieSync(cookies: SerializedCookieSyncEntry[], options?: { clear?: boolean; destination?: RequestDestination }): Promise<void>;
}

export interface SerializedCookieSyncEntry {
    url: string;
    cookie: string;
}

export interface Cookie {
    name: string;
    value: string;
    path?: string;
    expires?: number;
    maxAge?: number;
    domain?: string;
    hostOnly?: boolean;
    secure?: boolean;
    httpOnly?: boolean;
    sameSite?: string;
}

export interface CookieJar {
    setCookies(cookieString: string, url: URL): void;
    getCookies(url: URL, fromJs: boolean, sameSiteContext?: "strict" | "lax" | "cross-site"): string;
    load(cookies: string): void;
    dump(): string;
    clear(): void;
}

export interface HttpCachePluginOptions {
  cacheName?: string;
}

export declare class HttpCachePlugin extends ManagedPluginBaseLike {
  readonly cacheName: string;
  constructor(options?: HttpCachePluginOptions);
  bust(): Promise<boolean>;
}

export type UrlWatcherOptions = Record<string, never>;

export declare class UrlWatcherPlugin extends ManagedPluginBaseLike {
  constructor(onUrlChange: (url: string) => void, options?: UrlWatcherOptions);
}

export declare class CatchEscapedLinksPlugin extends ManagedPluginBaseLike {
  constructor(toLocation: (url: URL) => string | URL);
}

export type EventHandlerPluginOptions = {
  events?: string[];
};

export declare class EventHandlerPlugin extends ManagedPluginBaseLike {
  constructor(options?: EventHandlerPluginOptions);
  addEventToCapture(eventName: string): void;
  addEventListener<T extends Event>(
    target: EventTarget,
    eventName: string,
    listener: (e: T) => void
  ): void;
}

export type LinkHandlerPluginOptions = Record<string, never>;

export declare class LinkHandlerPlugin extends ManagedPluginBaseLike {
  constructor(onNewTab: (url: string) => void, options?: LinkHandlerPluginOptions);
}