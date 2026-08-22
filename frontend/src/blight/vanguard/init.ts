import init from "vanguard";
import { CONFIG } from "../config";

let initPromise: Promise<unknown> | null = null;

export function initVanguard(): Promise<unknown> {
  if (!initPromise) {
    initPromise = init({ module_or_path: CONFIG.vanguardWasm });
  }
  return initPromise;
}