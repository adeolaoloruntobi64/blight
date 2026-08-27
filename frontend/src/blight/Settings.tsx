import { useEffect, useState } from "react";
import { TRANSPORT_NAMES, type TransportName } from "./transports/defaults";
import { getTransportOptions, setTransportOptions, resetTransportOptions } from "./transports/options";
import type { IDBStore } from "./vanguard/idb";

// Should definently have more stuff, like all time stats, theme maybe, history

function TransportOptionsEditor({ store, name }: { store: IDBStore; name: TransportName }) {
    const [options, setOptions] = useState<Record<string, unknown> | null>(null);

    useEffect(() => {
        getTransportOptions<Record<string, unknown>>(store, name).then(setOptions);
    }, [name]);

    if (!options) return <div>Loading…</div>;

    const update = (key: string, value: string) => {
        const next = { ...options, [key]: value };
        setOptions(next);
        setTransportOptions(store, name, next as any);
    };

    return (
        <div style={{ border: "1px solid #ddd", borderRadius: 8, padding: 12, marginTop: 8 }}>
            {Object.entries(options).map(([key, value]) => (
                <label key={key} style={{ display: "block", marginBottom: 8 }}>
                    <span style={{ display: "block", fontSize: 12, color: "#666" }}>{key}</span>
                    <input
                        value={String(value)}
                        onChange={(e) => update(key, e.target.value)}
                        style={{ width: "100%", padding: 4 }}/>
                </label>
            ))}
            <button
                onClick={async () => {
                    await resetTransportOptions(store, name);
                    setOptions(await getTransportOptions(store, name));
                }}>
                Restore defaults
            </button>
        </div>
    );
}

export function SettingsPage({ store }: { store: IDBStore }) {
    const [active, setActive] = useState<TransportName>("epoxy");

    useEffect(() => {
        store.get<TransportName>("active-transport-name").then((v) => v && setActive(v));
    }, []);

    const selectTransport = async (name: TransportName) => {
        setActive(name);
        await store.put("active-transport-name", name);
    };

    return (
        <div style={{ padding: 24, maxWidth: 500 }}>
            <h1>Transport</h1>
            <p style={{ color: "#666", fontSize: 13 }}>
                Changing the transport takes effect the next time Blight loads. This tab won't switch live.
            </p>
            {TRANSPORT_NAMES.map((name) => (
                <label key={name} style={{ display: "block", margin: "8px 0" }}>
                    <input
                        type="radio"
                        name="transport"
                        checked={active === name}
                        onChange={() => selectTransport(name)}/>
                    {` ${name}`}
                </label>
            ))}
            <TransportOptionsEditor store={store} name={active} />
        </div>
    );
}