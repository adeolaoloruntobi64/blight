import { useEffect, useImperativeHandle, useRef } from "react";
import { VanguardPlugin } from "./vanguard/plugin";
import type { BlightContext } from "./boot";
import type { FrameLike } from "./types/scramjet-hooks";
import { TabMetadataPlugin, TabMetaAnnouncement } from "./tab-meta";
import { BLIGHT_SETTINGS_URL, isInternalUrl } from "./internal";
import { SettingsPage } from "./Settings";

export interface TabHandle {
    back(): void;
    forward(): void;
    reload(): void;
    go(url: string): void;
}

export type TabParams = {
    blight: BlightContext;
    url: string;
    active: boolean;
    announce: (m: TabMetaAnnouncement) => void;
    openNewTab: (url: string) => void;
    ref?: React.Ref<TabHandle>;
};

export function Tab(params: TabParams) {
    const containerRef = useRef<HTMLDivElement>(null);
    const frameRef = useRef<FrameLike | null>(null);
    // Whenever we look at this tab object / element, instead
    // of returning the actual element, return an object that exposes
    // these apis
    useImperativeHandle(params.ref, () => ({
        back: () => frameRef.current?.back(),
        forward: () => frameRef.current?.forward(),
        reload: () => frameRef.current?.reload(),
        go: (u: string) => frameRef.current?.go(u),
    }));

    useEffect(() => {
        if (isInternalUrl(params.url)) return;
        const iframeEl = document.createElement("iframe");
        iframeEl.className = "frame";
        iframeEl.style.cssText = "position:absolute; inset:0; width:100%; height:100%; border:none;";
        iframeEl.setAttribute(
            "sandbox",
            "allow-forms allow-modals allow-popups allow-presentation allow-same-origin allow-scripts allow-downloads"
        );
        const frame = params.blight.controller.createFrame(iframeEl, {
            plugins: [
                new VanguardPlugin(params.blight.VanguardRequest, params.blight.holder, params.blight.stats),
                new TabMetadataPlugin(params.announce),
                new $scramjetUtils.UrlWatcherPlugin((url) => params.announce({ url })),
                new $scramjetUtils.CatchEscapedLinksPlugin((url) => new URL(location.href)), // temporary
                // Should these be per tab or a singleton? Might be worth thinking about later
                new $scramjetUtils.HttpCachePlugin(),
                new $scramjetUtils.EventHandlerPlugin(),
                new $scramjetUtils.LinkHandlerPlugin((url) => params.openNewTab(url)),
            ],
        });
        frameRef.current = frame;
        containerRef.current?.appendChild(iframeEl);
        frame.go(params.url);
        // Thinking of adding eruda. Would need it to persist through navigation. Maybe an ErudaPlugin?
        return () => {
            const index = params.blight.controller.frames.indexOf(frame);
            if (index !== -1) params.blight.controller.frames.splice(index, 1);
            iframeEl.remove();
        };
    }, []);

    useEffect(() => {
        if (frameRef.current) {
             // The wrapper div now controls visibility, so always block
            frameRef.current.element.style.display = "block";
            if (params.active) {
                frameRef.current.element.focus();
            }
        }
    }, [params.active]);

    if (isInternalUrl(params.url))
        return (
            <div style={{ position: "absolute", inset: 0, display: params.active ? "block" : "none", overflow: "auto" }}>
                {params.url === BLIGHT_SETTINGS_URL && <SettingsPage store={params.blight.vstore.getStore()} />}
            </div>
        );

    return (
        <div
            ref={containerRef}
            style={{ position: "absolute", inset: 0, display: params.active ? "block" : "none" }}/>
    );
}