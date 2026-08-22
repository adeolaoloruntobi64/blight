import type { PluginBaseLike, ManagedPluginBaseLike, ControllerLike, ScramjetHeaders, CookieJar, HttpCachePlugin, UrlWatcherPlugin, CatchEscapedLinksPlugin, EventHandlerPlugin, LinkHandlerPlugin } from "./scramjet-hooks";
import type { BareResponse } from "@mercuryworkshop/proxy-transports";

declare global {
    var $scramjet: {
        Plugin: typeof PluginBaseLike;
        ScramjetHeaders: { fromRawHeaders(raw: unknown): ScramjetHeaders; fromNativeHeaders(native: Headers): ScramjetHeaders };
        CookieJar: new () => CookieJar;
        BareResponse: typeof BareResponse;
    };
    var $scramjetUtils: {
        ManagedPlugin: typeof ManagedPluginBaseLike;
        HttpCachePlugin: typeof HttpCachePlugin;
        UrlWatcherPlugin: typeof UrlWatcherPlugin;
        CatchEscapedLinksPlugin: typeof CatchEscapedLinksPlugin;
        EventHandlerPlugin: typeof EventHandlerPlugin;
        LinkHandlerPlugin: typeof LinkHandlerPlugin;
    };
    var $scramjetController: {
        Controller: typeof ControllerLike;
        VERSION: string;
        assertRuntimeScramjetVersion(): void;
    };
}

export { };