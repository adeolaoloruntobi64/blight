import { SendableUrlSpecificResources } from "./util";

export function inject(parsedInfo: {html: string, bodyCloseStartIndex: number, htmlCloseStartIndex: number}, urlres: SendableUrlSpecificResources): string {
    // Try to insert at the end tag of body. If we couldn't find that,
    // try to insert at the end tag of html. If we couldn't find that,
    // insert it at the end of the html
    const insertionIndex = parsedInfo.bodyCloseStartIndex !== -1
        ? parsedInfo.bodyCloseStartIndex
        : parsedInfo.htmlCloseStartIndex !== -1
            ? parsedInfo.htmlCloseStartIndex
            : parsedInfo.html.length - 1;

    let injectedCSS = '';
    let injectedJS = '';

    // `hide_selectors` is a set of any CSS selector on the page that should be hidden, i.e.
    /// styled as `{ display: none !important; }`.
    const hideSelectors = urlres.hide_selectors;
    if (hideSelectors.length)
        injectedCSS += `${hideSelectors.join(',')}{display:none!important;}`;

    /// Set of JSON-encoded procedural filters or filters with an action.
    const proceduralActions = urlres.procedural_actions;
    // I dunno what to do w/ this rn. Also inject MutationnObserver
        
    /// `injected_script` is the Javascript code for any scriptlets that should be injected into the page.
    const injectedScript = urlres.injected_script;
    if (injectedScript.length) injectedJS += injectedScript;  

    let newHtml = parsedInfo.html.slice(0, insertionIndex);
    if (injectedCSS.length) newHtml += `<style>${injectedCSS}</style>`;
    if (injectedJS.length) newHtml += `<script type="application/javascript">${injectedJS}</script>`;
    newHtml += parsedInfo.html.slice(insertionIndex);
    return newHtml;
}