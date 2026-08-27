export interface TabState {
    id: string;
    url: string;
    title: string;
    favicon: string | null;
    loading: boolean;
}

export interface TabBarProps {
    tabs: TabState[];
    activeId: string;
    onSelect: (id: string) => void;
    onClose: (id: string) => void;
    onNewTab: () => void;
}

export function TabBar({ tabs, activeId, onSelect, onClose, onNewTab }: TabBarProps) {
    return (
        <div style={{ display: "flex", alignItems: "center", height: 36, background: "#e8e8e8" }}>
            {tabs.map((tab) => (
                <div
                    key={tab.id}
                    onClick={() => onSelect(tab.id)}
                    style={{
                        display: "flex",
                        alignItems: "center",
                        gap: 6,
                        padding: "0 10px",
                        height: "100%",
                        maxWidth: 220,
                        background: tab.id === activeId ? "#fff" : "transparent",
                        borderRight: "1px solid #ccc",
                        cursor: "pointer",
                    }}>
                    {tab.favicon ? (
                        <img src={tab.favicon} width={16} height={16} alt="" />
                    ) : (
                        <div style={{ width: 16, height: 16, background: "#ccc", borderRadius: 2 }} />
                    )}
                    <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1, fontSize: 13 }}>
                        {tab.loading ? "Loading…" : tab.title}
                    </span>
                    <button
                        onClick={(e) => { e.stopPropagation(); onClose(tab.id); }}
                        style={{ border: "none", background: "none", cursor: "pointer", fontSize: 14, lineHeight: 1 }}>
                        ×
                    </button>
                </div>
            ))}
            <button onClick={onNewTab} style={{ padding: "0 10px", height: "100%", border: "none", background: "none", cursor: "pointer", fontSize: 18 }}>
                +
            </button>
        </div>
    );
}