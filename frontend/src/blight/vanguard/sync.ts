export class VanguardSyncChannel {
  #channel = new BroadcastChannel("vanguard-sync");

  onConfigChanged(callback: () => void) {
    this.#channel.addEventListener("message", (e) => {
      if (e.data.type === "config-changed") callback();
    });
  }

  announceConfigChanged() {
    this.#channel.postMessage({ type: "config-changed" });
  }

  onStatsUpdated(callback: (stats: { blocked: number; allowed: number }) => void) {
    this.#channel.addEventListener("message", (e) => {
      if (e.data.type === "stats-updated") callback(e.data.stats);
    });
  }

  announceStatsUpdated(stats: { blocked: number; allowed: number }) {
    this.#channel.postMessage({ type: "stats-updated", stats });
  }
}