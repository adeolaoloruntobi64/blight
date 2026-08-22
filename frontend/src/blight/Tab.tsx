import { useEffect, useRef } from "react";
import { VanguardPlugin } from "./vanguard/plugin";
import type { BlightContext } from "./boot";
import type { FrameLike, ManagedPluginBaseLike } from "./types/scramjet-hooks";

export function Tab({ blight, url, active }: { blight: BlightContext; url: string; active: boolean }) {
  const containerRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef<FrameLike | null>(null);

  useEffect(() => {
    const iframeEl = document.createElement("iframe");
    iframeEl.className = "frame";
    iframeEl.style.cssText = "position:absolute; inset:0; width:100%; height:100%; border:none;";
    //iframeEl.setAttribute("credentialless", "true");
    iframeEl.setAttribute(
      "sandbox",
      "allow-forms allow-modals allow-popups allow-presentation allow-same-origin allow-scripts allow-downloads"
    );
    $scramjetUtils
    const frame = blight.controller.createFrame(iframeEl, {
      plugins: [/*new VanguardPlugin(blight.holder, blight.stats)*/],
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