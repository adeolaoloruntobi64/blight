import type { FrameLike } from "./types/scramjet-hooks";

const scramjetUtils = () => globalThis.$scramjetUtils;

export interface TabMetaAnnouncement {
    url?: string;
    title?: string;
    favicon?: string | null;
    loading?: boolean;
}

export class TabMetadataPlugin extends scramjetUtils().ManagedPlugin {
    #announce: (meta: TabMetaAnnouncement) => void;

    constructor(announce: (meta: TabMetaAnnouncement) => void) {
        super("tab-metadata", []);
        this.#announce = announce;
    }

    install(frame: FrameLike) {
        super.install(frame);

        this.tap(frame.hooks.init.post, (context) => {
            if (!context.isTopLevel) return;
            const doc = context.window.document;
            this.#announce({ loading: true });
            const announceTitle = () => this.#announce({ title: doc.title || context.client.url });
            const announceFavicon = () => {
                const link = doc.querySelector<HTMLLinkElement>("link[rel~='icon']");
                this.#announce({ favicon: link?.href ?? null });
            };
            announceTitle();
            announceFavicon();
            // Whenever title or favicon changes, announce it
            new MutationObserver(announceTitle).observe(doc.querySelector("title") ?? doc.head, {
                childList: true, subtree: true, characterData: true,
            });
            new MutationObserver(announceFavicon).observe(doc.head, {
                childList: true, subtree: true, attributes: true, attributeFilter: ["href", "rel"],
            });
            // If the window is done loading, say so. Only emit this event once
            context.window.addEventListener("load", () => this.#announce({ loading: false }), { once: true });
            // On navigation, announce url change
            this.tap(context.client.hooks.lifecycle.navigate, (_ctx, props) => {
                this.#announce({ url: props.url, loading: true });
            });
        });
    }
}