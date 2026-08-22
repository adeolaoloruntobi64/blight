import { openDB, type IDBPDatabase } from "idb";

export type IDBItem<T> = { key: string; value: T };

export class IDBStore {
  #dbPromise: Promise<IDBPDatabase> | null = null;

  constructor(
    private readonly storeName: string,
    private readonly storeVersion: number,
    private readonly objectStore: string
  ) {}

  private getDB(): Promise<IDBPDatabase> {
    if (!this.#dbPromise) {
      this.#dbPromise = openDB(this.storeName, this.storeVersion, {
        upgrade: (db) => {
          if (!db.objectStoreNames.contains(this.objectStore)) {
            db.createObjectStore(this.objectStore, { keyPath: "key" });
          }
        },
      });
    }
    return this.#dbPromise;
  }

  async get<T>(key: string): Promise<T | undefined> {
    const db = await this.getDB();
    const entry: IDBItem<T> | undefined = await db.get(this.objectStore, key);
    return entry?.value;
  }

  async put<T>(key: string, value: T): Promise<void> {
    const db = await this.getDB();
    await db.put(this.objectStore, { key, value });
  }

  /** One transaction, many keys — this is the batching you asked for. */
  async putMany(entries: Record<string, unknown>): Promise<void> {
    const db = await this.getDB();
    const tx = db.transaction(this.objectStore, "readwrite");
    await Promise.all(
      Object.entries(entries)
        .filter(([, v]) => v !== undefined)
        .map(([key, value]) => tx.store.put({ key, value }))
    );
    await tx.done;
  }

  async getMany<T = unknown>(keys: string[]): Promise<Record<string, T | undefined>> {
    const db = await this.getDB();
    const tx = db.transaction(this.objectStore, "readonly");
    const results = await Promise.all(keys.map((k) => tx.store.get(k) as Promise<IDBItem<T> | undefined>));
    await tx.done;
    return Object.fromEntries(keys.map((k, i) => [k, results[i]?.value]));
  }

    async increment(key: string, delta: number): Promise<number> {
        const db = await this.getDB();
        const tx = db.transaction(this.objectStore, "readwrite");
        const current = (await tx.store.get(key))?.value ?? 0;
        const next = current + delta;
        await tx.store.put({ key, value: next });
        await tx.done;
        return next;
    }
}