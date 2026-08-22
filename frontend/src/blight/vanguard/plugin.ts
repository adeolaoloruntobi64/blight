import { VanguardRequest } from "vanguard";
import type { FetchPreresponseContext, FetchPreresponseProps, FetchRequestContext, FetchRequestProps, FrameLike } from "../types/scramjet-hooks";
import { Element, Text } from "domhandler";
import * as DomUtils from "domutils";
import type { DomHandler } from "domhandler";
import type { VanguardHandle } from "./handle";
import type { StatsTracker } from "./stats";
import { mapDestination } from "./dest";

const scramjetUtils = () => globalThis.$scramjetUtils;

export class VanguardPlugin extends scramjetUtils().ManagedPlugin {
    #holder: VanguardHandle;
    #stats: StatsTracker;

    constructor(holder: VanguardHandle, stats: StatsTracker) {
        super("vanguard", []);
        this.#holder = holder;
        this.#stats = stats;
    }

    install(frame: FrameLike) {
        super.install(frame);

        this.tap(frame.hooks.fetch.request, (context, props) => {
            this.#applyRequestWithExclude(context, props);
        });

        this.tap(frame.hooks.fetch.preresponse, (context, props) => {
            this.#applyCspDirectives(context, props);
        });

        this.tap(frame.fetchHandler.hooks.rewriter.html.pre, (htmlCtx, htmlProps) => {
            const url = htmlCtx.meta.base.href;
            this.#applyCosmeticsPre(htmlCtx.handler, url);
        });

        this.tap(frame.hooks.init.post, (context) => {
            this.tap(context.client.hooks.lifecycle.navigate, (context2, props2) => {
                this.#applyCosmeticsLive(context.window.document, props2.url);
            });
        });
    }

    #applyRequestWithExclude(context: FetchRequestContext, props: FetchRequestProps) {
        const match = this.#holder.exclude.matchHost(props.url.toString());
        if (match) return;

        const req = new VanguardRequest(
            props.url.href,
            context.parsed.clientUrl?.href ?? props.url.href,
            context.parsed.destination ? mapDestination(context.parsed.destination) : "other",
            props.init.method ?? "GET"
        );
        const result = this.#holder.engine.check_network_request(req);
        const shouldBlock = result.filter !== undefined && (result.important || result.exception === undefined);

        if (shouldBlock) {
            props.earlyResponse = new Response("", { status: 204 });
            this.#stats.recordBlocked();
        } else {
            this.#stats.recordAllowed();
        }
    }

    #applyCspDirectives(context: FetchPreresponseContext, props: FetchPreresponseProps) {
        const destination = context.parsed.destination ? mapDestination(context.parsed.destination) : "other";
        if (destination !== "document") return;

        const match = this.#holder.exclude.matchHost(context.parsed.url.toString());
        if (match) return;

        const req = new VanguardRequest(
            context.parsed.url.href,
            context.parsed.url.href,
            destination,
            context.request?.method ?? "GET"
        );
        const csp = this.#holder.engine.get_csp_directives(req);
        if (!csp) return;

        const existing = props.response.headers?.get?.("content-security-policy");
        props.response.headers?.set?.(
            "content-security-policy",
            existing ? `${existing}, ${csp}` : csp
        );
    }

    #applyCosmeticsLive(doc: Document, url: string) {
        const resources = this.#holder.engine.url_cosmetic_resources(url);
        let styleEl = doc.getElementById("__vanguard_hide") as HTMLStyleElement | null;
        if (!styleEl) {
            styleEl = doc.createElement("style");
            styleEl.id = "__vanguard_hide";
            doc.head.appendChild(styleEl);
        }
        if (resources.generichide) {
            styleEl.textContent = `${resources.hide_selectors.join(", ")} { display:none!important }`;
            return;
        }
        const classes = [...new Set(Array.from(doc.querySelectorAll("[class]")).flatMap(
            (el) => el.className.split(/\s+/).filter(Boolean)
        ))];
        const ids = [...new Set(Array.from(doc.querySelectorAll("[id]")).map((el) => el.id))];
        const extra = this.#holder.engine.hidden_class_id_selectors(classes, ids, resources.exceptions);
        styleEl.textContent = `${[...resources.hide_selectors, ...extra].join(", ")} { display:none!important }`;
    }

    #applyCosmeticsPre(handler: DomHandler, url: string) {
        const resources = this.#holder.engine.url_cosmetic_resources(url);
        if (resources.generichide) return this.#injectStyle(handler, resources.hide_selectors);

        const { classes, ids } = this.#collectClassesAndIds(handler);
        const extra = this.#holder.engine.hidden_class_id_selectors(classes, ids, resources.exceptions);
        this.#injectStyle(handler, [...resources.hide_selectors, ...extra]);
        if (resources.injected_script) this.#injectScript(handler, resources.injected_script);
    }

    #findHead(handler: DomHandler): Element | undefined {
        return DomUtils.findOne((el) => el.type === "tag" && el.name === "head", handler.dom, true) as Element | undefined;
    }
    #injectStyle(handler: DomHandler, selectors: string[]) {
        if (selectors.length === 0) return;
        const head = this.#findHead(handler);
        if (!head) return;
        DomUtils.appendChild(head, new Element("style", {}, [new Text(`${selectors.join(", ")} { display: none !important; }`)]));
    }
    #injectScript(handler: DomHandler, script: string) {
        const head = this.#findHead(handler);
        if (!head) return;
        DomUtils.prependChild(head, new Element("script", {}, [new Text(script)]));
    }
    #collectClassesAndIds(handler: DomHandler): { classes: string[]; ids: string[] } {
        const classes = new Set<string>(), ids = new Set<string>();
        for (const el of DomUtils.findAll(() => true, handler.dom) as Element[]) {
            el.attribs?.class?.split(/\s+/).filter(Boolean).forEach((c) => classes.add(c));
            if (el.attribs?.id) ids.add(el.attribs.id);
        }
        return { classes: [...classes], ids: [...ids] };
    }
}