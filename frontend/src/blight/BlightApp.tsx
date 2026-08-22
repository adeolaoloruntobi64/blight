import { useEffect, useState } from "react";
import { bootBlight, type BlightContext } from "./boot";
import { Tab } from "./Tab";
import { initVanguard } from "./vanguard/init";

interface TabState { id: string; url: string }

function BlightApp() {
  const [blight, setBlight] = useState<BlightContext | null>(null);
  const [tabs, setTabs] = useState<TabState[]>([{ id: crypto.randomUUID(), url: "https://www.google.com" }]);
  const [activeId, setActiveId] = useState(tabs[0].id);

  useEffect(() => { initVanguard().then(_ => bootBlight().then(setBlight).catch(console.error)) }, []);

  if (!blight) return <div>Loading…</div>;

  return (
    <div style={{ position: "relative", width: "100vw", height: "100vh" }}>
      {tabs.map((tab) => (
        <Tab key={tab.id} blight={blight} url={tab.url} active={tab.id === activeId} />
      ))}
    </div>
  );
}

export default BlightApp;