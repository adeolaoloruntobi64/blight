import { CONFIG } from "./config";

let loaded: Promise<void> | null = null;

function loadScript(src: string): Promise<void> {
  return new Promise((res, rej) => {
    const el = document.createElement("script");
    el.src = src;
    el.onload = () => res();
    el.onerror = () => rej(new Error(`Failed to load ${src}`));
    document.head.appendChild(el);
  });
}

export function loadScramjetScripts(): Promise<void> {
  if (!loaded) {
    loaded = (async () => {
        await loadScript(CONFIG.scramjet);
        await loadScript(CONFIG.scramjetControllerApi);
        await loadScript(CONFIG.scramjetUtils);
    })();
  }
  return loaded;
}