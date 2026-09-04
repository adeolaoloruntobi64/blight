import type { VanguardRequest as VanguardRequestClass } from "vanguard";
import type {
    FetchPreresponseContext,
    FetchPreresponseProps,
    FetchRequestContext,
    FetchRequestProps,
    FrameLike,
} from "../types/scramjet-hooks";
import { Element, Text } from "domhandler";
import * as DomUtils from "domutils";
import type { DomHandler } from "domhandler";
import type { VanguardHandle } from "./handle";
import type { StatsTracker } from "./stats";
import { mapDestination } from "./dest";
import selectMutateContents from "./observer?ts-to-js-str";

const PRE_JS_SUFFIX = "vanguard-pre-js-inject-resources";
const PRE_CSS_SUFFIX = "vanguard-pre-css-inject-selectors";
const PRE_JS_CSS_SUFFIX = "vanguard-pre-js-inject-selectors";
const LIVE_CSS_SUFFIX = "vanguard-live-css-inject-selectors";
const LIVE_JS_CSS_SUFFIX = "vanguard-live-js-inject-selectors";

const gensfx = () => "-" + crypto.randomUUID(); // to generate a suffix
const scramjetUtils = () => globalThis.$scramjetUtils;

export class VanguardPlugin extends scramjetUtils().ManagedPlugin {
    #holder: VanguardHandle;
    #stats: StatsTracker;
    #localAllowed: number; // Per-tab stats are fleeting. All-time stats are permanent
    #localBlocked: number;
    VanguardRequest: typeof VanguardRequestClass;

    constructor(VanguardRequest: typeof VanguardRequestClass, holder: VanguardHandle, stats: StatsTracker) {
        super("vanguard", []);
        this.#holder = holder;
        this.#stats = stats;
        this.#localAllowed = 0;
        this.#localBlocked = 0;
        this.VanguardRequest = VanguardRequest;
    }

    install(frame: FrameLike) {
        super.install(frame);

        this.tap(frame.hooks.fetch.request, (context, props) => {
            this.#applyRequestWithExclude(context, props);
        });

        this.tap(frame.hooks.fetch.preresponse, (context, props) => {
            this.#applyCspDirectives(context, props);
        });

        this.tap(frame.fetchHandler.hooks.rewriter.html.pre, (context, props) => {
            this.#applyCosmeticsPre(context.handler, context.meta.base.href);
        });

        this.tap(frame.hooks.init.post, (context, props) => {
            this.tap(context.client.hooks.lifecycle.navigate, (context2, props2) => {
                this.#applyCosmeticsLive(context.window.document, props2.url);
            });
        });
    }

    #applyRequestWithExclude(context: FetchRequestContext, props: FetchRequestProps) {
        const match = this.#holder.exclude.matchHost(props.url.hostname);
        if (match) return;
        const destination = mapDestination(context.parsed.destination);
        const req = new this.VanguardRequest(
            props.url.href,
            context.parsed.clientUrl?.href ?? props.url.href,
            destination,
            props.init.method ?? "GET"
        );
        const result = this.#holder.engine.check_network_request(req);
        if (result.filter === undefined) {
            this.#localAllowed++;
            this.#stats.recordAllowed();
            return;
        }
        if (result.important) {
            this.#localBlocked++;
            this.#stats.recordBlocked();
            props.earlyResponse = new Response(null, { status: 204 });
            return;
        }
        if (result.exception !== undefined) {
            this.#localAllowed++;
            this.#stats.recordAllowed();
            return;
        }
        if (result.redirect) {
            this.#localAllowed++;
            this.#stats.recordAllowed();
            props.earlyResponse = this.#dataUrlToResponse(result.redirect);
            return;
        }
        if (result.rewritten_url) {
            this.#localAllowed++;
            this.#stats.recordAllowed();
            props.url = new URL(result.rewritten_url);
            return;
        }
        this.#localBlocked++;
        this.#stats.recordBlocked();
        props.earlyResponse = new Response(null, { status: 204 });
    }

    #applyCspDirectives(context: FetchPreresponseContext, props: FetchPreresponseProps) {
        const destination = context.parsed.destination ? mapDestination(context.parsed.destination) : "other";
        if (destination !== "document") return;

        const match = this.#holder.exclude.matchHost(context.parsed.url.toString());
        if (match) return;

        const req = new this.VanguardRequest(
            context.parsed.url.href,
            context.parsed.url.origin,
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

    #applyCosmeticsPre(handler: DomHandler, url: string) {
        const resources = this.#holder.engine.url_cosmetic_resources(url);
        const { urlRules, plainSelectors } = this.#splitUrlSelectors(resources.hide_selectors);

        if (!resources.generichide) {
            const { classes, ids } = this.#collectClassesAndIds(handler);
            plainSelectors.push(...this.#holder.engine.hidden_class_id_selectors(classes, ids, resources.exceptions));
        }
        // We can use pure CSS selectors for non url-based selectors
        if (plainSelectors.length > 0)
            this.#injectStyle(handler, PRE_CSS_SUFFIX + gensfx(), plainSelectors);
        if (urlRules.length > 0) {
            // We need JS for url-based rules because scramjet rewrites the url, and we can't
            // invoke scramjet in the plugin because the url might be relative to the origin,
            // or absolute, and scramjet adds some other tags to urls.
            const hstr = `hideSelectors(document,${JSON.stringify(urlRules.join(","))})`;
            const script = `${selectMutateContents}${hstr}`;
            this.#injectScript(handler, PRE_JS_CSS_SUFFIX + gensfx(), script);
        }
        if (resources.injected_script)
            this.#injectScript(handler, PRE_JS_SUFFIX + gensfx(), resources.injected_script);
    }

    #applyCosmeticsLive(doc: Document, url: string) {
        const resources = this.#holder.engine.url_cosmetic_resources(url);
        const { urlRules, plainSelectors } = this.#splitUrlSelectors(resources.hide_selectors);

        if (!resources.generichide) {
            const classes = Array.from(doc.querySelectorAll("[class]")).flatMap((el) => [...el.classList]);
            const ids = Array.from(doc.querySelectorAll("[id]")).map((el) => el.id);
            plainSelectors.push(...this.#holder.engine.hidden_class_id_selectors(classes, ids, resources.exceptions));
        }
        // We are injecting into a live html document instead of a DomUtils tree
        if (plainSelectors.length > 0) {
            const styleEl = doc.createElement("style");
            styleEl.id = LIVE_CSS_SUFFIX + gensfx();
            styleEl.textContent = plainSelectors.length ? `${plainSelectors.join(", ")} { display:none!important }` : "";
            doc.head.prepend(styleEl);
        }
        if (urlRules.length > 0) {
            const scriptEl = doc.createElement("script");
            scriptEl.id = LIVE_JS_CSS_SUFFIX + gensfx();
            const hstr = `hideSelectors(document,${JSON.stringify(urlRules.join(","))})`;
            const script = `${selectMutateContents}${hstr}`;
            scriptEl.textContent = script;
            scriptEl.type = "module"
            doc.head.prepend(scriptEl);
        }
    }

    #findHead(handler: DomHandler): Element | null {
        return DomUtils.findOne((el) => el.type === "tag" && el.name === "head", handler.dom, true);
    }
    
    #injectStyle(handler: DomHandler, id: string, selectors: string[]) {
        if (selectors.length === 0) return;
        const head = this.#findHead(handler);
        if (!head) return;
        // Since I do this in injectScript, might as well do it here
        const scriptElement = new Element("style", { id });
        DomUtils.prependChild(scriptElement, new Text(`${selectors.join(", ")} { display: none !important; }`));
        DomUtils.prependChild(head, scriptElement);
    }

    #injectScript(handler: DomHandler, id: string, script: string) {
        const head = this.#findHead(handler);
        if (!head) return;
        // It is very, VERY important to assert the parent-child relationship in BOTH directions
        // Previously, I only prepended the element with a Text child to the head. The problem was,
        // doing new Element(..., [new Text(...)]) did not set the parent of text to be the element.
        // This meant when parsing the tree, the parser wouldn't know that the text is for a script,
        // and it would encode special characters (< & > became &amp; >gt; <lt;) assuming it was plain text.
        const scriptElement = new Element("script", {id , type: "module" });
        DomUtils.prependChild(scriptElement, new Text(script));
        DomUtils.prependChild(head, scriptElement);
    }

    #collectClassesAndIds(handler: DomHandler): { classes: string[]; ids: string[] } {
        const classes = new Set<string>(), ids = new Set<string>();
        for (const el of DomUtils.findAll(() => true, handler.dom)) {
            el.attribs?.class?.split(/\s+/).filter(Boolean).forEach((c) => classes.add(c));
            if (el.attribs?.id)
                ids.add(el.attribs.id);
        }
        return { classes: [...classes], ids: [...ids] };
    }
    
    #splitUrlSelectors(hide_selectors: string[]): { urlRules: string[]; plainSelectors: string[]; } {
        const urlRules: string[] = [];
        const plainSelectors: string[] = [];
        const urlregex = /\[(?:href|src|srcset|action|poster|data|cite|formaction)[\^*$]?=/
        for (const sel of hide_selectors) {
            if (urlregex.test(sel))
                urlRules.push(sel);
            else
                plainSelectors.push(sel);
        }
        return { urlRules, plainSelectors };
    }

    #dataUrlToResponse(dataUrl: string): Response {
        const [meta, data] = dataUrl.slice(5).split(",", 2); // strip "data:"
        const isBase64 = meta.endsWith(";base64");
        const mime = meta.replace(";base64", "") || "text/plain";
        const bytes = isBase64
            ? Uint8Array.from(atob(data), (c) => c.charCodeAt(0))
            : new TextEncoder().encode(decodeURIComponent(data));
        return new Response(bytes, { headers: { "content-type": mime } });
    }
}