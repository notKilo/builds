import fs from "node:fs";
import type { Store } from "./store.ts";

/** Write atomically: temp file in the same directory, then rename. */
export function saveSnapshot(store: Store, file: string): void {
  const tmp = `${file}.tmp`;
  fs.writeFileSync(tmp, JSON.stringify(store.snapshot()));
  fs.renameSync(tmp, file);
}

/** Load a snapshot if the file exists. Returns true when data was restored. */
export function loadSnapshot(store: Store, file: string): boolean {
  let text: string;
  try {
    text = fs.readFileSync(file, "utf8");
  } catch (e) {
    if ((e as NodeJS.ErrnoException).code === "ENOENT") return false;
    throw e;
  }
  store.restore(JSON.parse(text));
  return true;
}

/** Save every intervalMs; returns a stop function that does a final save. */
export function autosave(store: Store, file: string, intervalMs: number): () => void {
  const timer = setInterval(() => saveSnapshot(store, file), intervalMs);
  timer.unref();
  return () => {
    clearInterval(timer);
    saveSnapshot(store, file);
  };
}
