import { useEffect, useRef, useState } from "react";
import { bootBlight, type BlightContext } from "./boot";
import { Tab, TabHandle } from "./Tab";
import { TabBar, TabState } from "./TabBar";
import { NavBar } from "./NavBar";
import { BLIGHT_SETTINGS_URL } from "./internal";

function makeTab(url: string): TabState {
  if (url === BLIGHT_SETTINGS_URL)
    return { id: crypto.randomUUID(), url, title: "Settings", favicon: null, loading: false };
  return { id: crypto.randomUUID(), url, title: "New Tab", favicon: null, loading: false };
}

function BlightApp() {
    const [blight, setBlight] = useState<BlightContext | null>(null);
    const [tabs, setTabs] = useState<TabState[]>([makeTab("https://www.google.com")]);
    const [activeId, setActiveId] = useState(tabs[0].id);
    const tabHandles = useRef(new Map<string, TabHandle>());

    useEffect(() => { bootBlight().then(setBlight).catch(console.error) }, []);

    if (!blight) return <div>Loading…</div>;

    const updateTab = (id: string, patch: Partial<TabState>) => {
        setTabs((prev) => prev.map((t) => (t.id === id ? { ...t, ...patch } : t)));
    };

    const openTab = (url = "https://www.google.com") => {
        const tab = makeTab(url);
        setTabs((prev) => [...prev, tab]);
        setActiveId(tab.id);
    };

    const closeTab = (id: string) => {
        setTabs((prev) => {
            // If I use a counter instead fo crypto, I could use binary search
            const index = prev.findIndex((t) => t.id === id);
            if (index === -1)
                return prev;
            const next = prev.filter((t) => t.id !== id);
            tabHandles.current.delete(id);
            if (activeId === id) {
                const newActive = next[index] ?? next[index - 1];
                if (newActive)
                    setActiveId(newActive.id);
            }
            // Can't have no tabs open
            return next.length > 0 ? next : [makeTab("https://www.google.com")];
        });
    };
    const activeTab = tabs.find((t) => t.id === activeId);
    const activeHandle = tabHandles.current.get(activeId);
    // Can be used across multiple frames
    const httpCache = new $scramjetUtils.HttpCachePlugin();
    return (
        <div style={{ display: "flex", flexDirection: "column", width: "100vw", height: "100vh" }}>
            <TabBar
                tabs={tabs}
                activeId={activeId}
                onSelect={setActiveId}
                onClose={closeTab}
                onNewTab={() => openTab()}/>
            <NavBar activeTab={activeTab} activeHandle={activeHandle} openNewTab={openTab} />
            <div style={{ position: "relative", flex: 1 }}>
                {tabs.map((tab) => (
                    <Tab
                        key={tab.id}
                        blight={blight}
                        url={tab.url}
                        active={tab.id === activeId}
                        announce={(m) => updateTab(tab.id, m)}
                        openNewTab={openTab}
                        httpCache={httpCache}
                        ref={(handle) => {
                            handle
                                ? tabHandles.current.set(tab.id, handle)
                                : tabHandles.current.delete(tab.id)
                        }}/>
                ))}
            </div>
        </div>
    );
}

export default BlightApp;