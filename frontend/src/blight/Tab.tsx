import { useEffect, useRef } from "react";
import { VanguardPlugin } from "./vanguard/plugin";
import type { BlightContext } from "./boot";
import type { FrameLike } from "./types/scramjet-hooks";

export function Tab({ blight, url, active }: { blight: BlightContext; url: string; active: boolean }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<FrameLike | null>(null);

  useEffect(() => {
    const iframeEl = document.createElement("iframe");
    iframeEl.className = "frame";
    iframeEl.style.cssText = "position:absolute; inset:0; width:100%; height:100%; border:none;";
    iframeEl.setAttribute(
      "sandbox",
      "allow-forms allow-modals allow-popups allow-presentation allow-same-origin allow-scripts allow-downloads"
    );
    const frame = blight.controller.createFrame(iframeEl, {
      plugins: [
        new VanguardPlugin(blight.holder, blight.stats),
        // new $scramjetUtils.UrlWatcherPlugin((url) => setTabUrl(tab.id, url)), think of this
        new $scramjetUtils.CatchEscapedLinksPlugin((url) => new URL(location.href)), // temporary
        new $scramjetUtils.HttpCachePlugin(),
        new $scramjetUtils.EventHandlerPlugin(),
        // new $scramjetUtils.LinkHandlerPlugin((url) => openNewTab(url)), also think of this
      ],
    });
    frameRef.current = frame;
    containerRef.current?.appendChild(iframeEl);
    frame.go(url);

    return () => {
      const index = blight.controller.frames.indexOf(frame);
      if (index !== -1) blight.controller.frames.splice(index, 1);
      iframeEl.remove();
    };
  }, []);

  useEffect(() => {
    if (frameRef.current) frameRef.current.element.style.display = active ? "block" : "none";
  }, [active]);

  return <div ref={containerRef} style={{ position: "absolute", inset: 0 }} />;
}