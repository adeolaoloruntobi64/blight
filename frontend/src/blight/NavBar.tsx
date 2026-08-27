import { useEffect, useState } from "react";
import type { TabHandle } from "./Tab";
import { BLIGHT_SETTINGS_URL } from "./internal";

interface NavBarProps {
    activeTab: { url: string; loading: boolean } | undefined;
    activeHandle: TabHandle | undefined;
    openNewTab: (url?: string) => void
}

function ThreeBarMenu({ openNewTab } : { openNewTab: (url?: string) => void }) {
    const [open, setOpen] = useState(false);

    return (
        <div style={{ position: "relative" }}>
            <button onClick={() => setOpen((o) => !o)}>☰</button>
            {open && (
                <div style={{ position: "absolute", right: 0, top: "100%", background: "#fff", border: "1px solid #ccc", borderRadius: 6, minWidth: 140, zIndex: 10 }}>
                    <button
                        style={{ display: "block", width: "100%", textAlign: "left", padding: 8 }}
                        onClick={() => { /* history? */ setOpen(false); }}>History</button>
                    <button
                        style={{ display: "block", width: "100%", textAlign: "left", padding: 8 }}
                        onClick={() => { setOpen(false); openNewTab() }}>New Tab</button>
                    <button
                        style={{ display: "block", width: "100%", textAlign: "left", padding: 8 }}
                        onClick={() => { setOpen(false); openNewTab(BLIGHT_SETTINGS_URL); }}>Settings</button>
                    <button
                        style={{ display: "block", width: "100%", textAlign: "left", padding: 8 }}
                        onClick={() => { /* downloads? */ setOpen(false); }}>Downloads</button>
                </div>
            )}
        </div>
    );
}

export function NavBar({ activeTab, activeHandle, openNewTab }: NavBarProps) {
    const [draft, setDraft] = useState(activeTab?.url ?? "");
    const [editing, setEditing] = useState(false);

    // Change the address bar text to the "real" URL whenever it changes under the hood, like
    // on navigation or redirect, but only while the user isn't actively typing.
    useEffect(() => {
        if (!editing) setDraft(activeTab?.url ?? "");
    }, [activeTab?.url, editing]);

    const navigate = (raw: string) => {
        let target = raw.trim();
        if (!target) return;
        if (!/^[a-z][a-z0-9+.-]*:\/\//i.test(target)) {
            target = target.includes(".") && !target.includes(" ")
                ? `https://${target}`
                : `https://www.google.com/search?q=${encodeURIComponent(target)}`;
        }
        activeHandle?.go(target);
        setEditing(false);
    };
    // Replace chars with images later
    return (
        <div style={{ display: "flex", alignItems: "center", gap: 6, padding: "6px 10px", background: "#f1f1f1" }}>
            <button onClick={() => activeHandle?.back()} title="Back">←</button>
            <button onClick={() => activeHandle?.forward()} title="Forward">→</button>
            <button onClick={() => activeHandle?.reload()} title="Reload">{activeTab?.loading ? "×" : "⟳"}</button>
            <input
                value={draft}
                onFocus={() => setEditing(true)}
                onChange={(e) => setDraft(e.target.value)}
                onBlur={() => setEditing(false)}
                onKeyDown={(e) => {
                    if (e.key === "Enter")
                        navigate(draft);
                    if (e.key === "Escape") {
                        setDraft(activeTab?.url ?? "");
                        (e.target as HTMLInputElement).blur();
                    }
                }}
                style={{ flex: 1, padding: "4px 10px", borderRadius: 16, border: "1px solid #ccc" }}
            />
            <ThreeBarMenu openNewTab={openNewTab} />
        </div>
    );
}