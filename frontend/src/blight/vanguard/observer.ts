type TrueDocument = Document & {
    $vanguard$observer: MutationObserver,
    $vanguard$allselectors: string
}

function hideSingleElement(elem: HTMLElement) {
    elem.style.setProperty("display", "none", "important");
}

function hideElementAndChildren(root: Element, cselectors: string) {
    if (root.matches(cselectors))
        hideSingleElement(root as any);
    root.querySelectorAll(cselectors).forEach(hideSingleElement as any);
};

// Technically, since we never import this function from elsewhere, there is no need to
// export it. But it doesn't hurt to add it
export function hideSelectors(doc: TrueDocument, selectors: string) {
    if (!doc.$vanguard$allselectors)
        doc.$vanguard$allselectors = selectors;
    else
        doc.$vanguard$allselectors += "," + selectors;
    hideElementAndChildren(doc.documentElement, selectors);
    if (doc.$vanguard$observer)
        return;
    const observer = new MutationObserver(mutations => {
        for (const mutation of mutations)
            for (const node of mutation.addedNodes)
                if (node.nodeType === Node.ELEMENT_NODE)
                    hideElementAndChildren(node as any, doc.$vanguard$allselectors);
    });
    observer.observe(doc.documentElement, { childList: true, subtree: true });
    doc.$vanguard$observer = observer;
}