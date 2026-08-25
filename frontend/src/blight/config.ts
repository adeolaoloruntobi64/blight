export const CONFIG = {
    prefix: "/blight/~/",
    sw: "/blight/sw.js",
    swScope: "/blight/",
    shellCachePrefix: "blight-shell-",
    shellCache: "blight-shell-v1",
    runtimeRoots: [
        "/deps/blight/transports/",
        "/deps/blight/scramjet/",
        "/deps/blight/vanguard/",
    ],

    bare: "/deps/blight/transports/bare.js",
    epoxy: "/deps/blight/transports/epoxy.js",
    epoxyWasm: "/deps/blight/transports/epoxy.wasm",
    libcurl: "/deps/blight/transports/libcurl.js",
    libcurlWasm: "/deps/blight/transports/libcurl.wasm",

    bareV1Url:  location.protocol + "//" + location.host + "/bare/v1/",
    bareV2Url:  location.protocol + "//" + location.host + "/bare/v2/",
    bareV3Url:  location.protocol + "//" + location.host + "/bare/v3/",
    wispV1Url:  (location.protocol == "https:" ? "wss:" : "ws:") + "//" + location.host + "/wisp/v1/",
    wispV2Url:  (location.protocol == "https:" ? "wss:" : "ws:") + "//" + location.host + "/wisp/v2/",
    wsproxyUrl: (location.protocol == "https:" ? "wss:" : "ws:") + "//" + location.host + "/wsproxy/",

    scramjet: "/deps/blight/scramjet/scramjet.js",
    scramjetWasm: "/deps/blight/scramjet/scramjet.wasm",
    scramjetControllerApi: "/deps/blight/scramjet/controller.api.js",
    scramjetUtils: "/deps/blight/scramjet/scramjet-utils.js",
    scramjetControllerInject: "/deps/blight/scramjet/controller.inject.js",

    vanguard: "/deps/blight/vanguard/vanguard.js",
    vanguardWasm: "/deps/blight/vanguard/vanguard.wasm",
    assetsJson: "/deps/blight/vanguard/assets.json"
}