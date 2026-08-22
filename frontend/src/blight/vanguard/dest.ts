const DESTINATION_MAP: Partial<Record<RequestDestination, string>> = {
  document: "document",
  frame: "sub_frame",
  iframe: "sub_frame",
  script: "script",
  style: "stylesheet",
  image: "image",
  font: "font",
  audio: "media",
  video: "media",
  object: "object",
  embed: "other",
  worker: "other",
  sharedworker: "other",
  paintworklet: "other",
  audioworklet: "other",
  track: "other",
  report: "other",
  xslt: "other",
  manifest: "other",
};

// navigator.sendBeacon() should map to Ping, not Xmlhttprequest. The Fetch spec gives beacon calls
// the same empty destination as fetch/XHR, so there's no way to tell them apart from destination alone.
// Not fixable without patching Scramjet's client-side beacon interception to tag it separately. It's not
// too important though
export function mapDestination(destination: RequestDestination): string {
  if (destination === "") return "xhr";
  return DESTINATION_MAP[destination] ?? "other";
}