import type { VanguardEngine, VanguardExclusionStore } from "vanguard";

export class VanguardHandle {
  #engine: VanguardEngine;
  #exclude: VanguardExclusionStore;

  constructor(engine: VanguardEngine, exclude: VanguardExclusionStore) {
    this.#engine = engine;
    this.#exclude = exclude;
  }

  get engine(): VanguardEngine { return this.#engine; }
  get exclude(): VanguardExclusionStore { return this.#exclude; }

  replaceEngine(engine: VanguardEngine): void {
    this.#engine = engine;
  }

  replaceExclude(exclude: VanguardExclusionStore): void {
    this.#exclude = exclude;
  }
}