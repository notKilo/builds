import { readFile, writeFile } from "node:fs/promises";
import { checkUrls, type HttpOptions, type UrlResult } from "./http.ts";

interface Entry {
  ok: boolean;
  status?: number;
  error?: string;
  checkedAt: number; // epoch ms
}

/** JSON-file cache of URL check results with a time-to-live. */
export class UrlCache {
  private entries = new Map<string, Entry>();

  private readonly file: string | null;
  private readonly ttlMs: number;
  private readonly now: () => number;

  constructor(file: string | null, ttlMs: number, now: () => number = Date.now) {
    this.file = file;
    this.ttlMs = ttlMs;
    this.now = now;
  }

  async load(): Promise<void> {
    if (!this.file) return;
    try {
      const data = JSON.parse(await readFile(this.file, "utf8")) as Record<string, Entry>;
      this.entries = new Map(Object.entries(data));
    } catch {
      this.entries = new Map(); // missing or corrupt cache: start fresh
    }
  }

  async save(): Promise<void> {
    if (!this.file) return;
    // Drop expired entries so the file doesn't grow forever.
    for (const [url, e] of this.entries) if (!this.fresh(e)) this.entries.delete(url);
    await writeFile(this.file, JSON.stringify(Object.fromEntries(this.entries), null, 1));
  }

  get(url: string): UrlResult | undefined {
    const e = this.entries.get(url);
    if (!e || !this.fresh(e)) return undefined;
    return { url, ok: e.ok, status: e.status, error: e.error };
  }

  set(result: UrlResult): void {
    const { url, ok, status, error } = result;
    this.entries.set(url, { ok, status, error, checkedAt: this.now() });
  }

  private fresh(e: Entry): boolean {
    return this.now() - e.checkedAt < this.ttlMs;
  }
}

/** Like checkUrls, but reuses fresh cached results and records new ones. */
export async function checkUrlsCached(
  urls: string[],
  opts: HttpOptions,
  cache: UrlCache,
): Promise<Map<string, UrlResult>> {
  const results = new Map<string, UrlResult>();
  const misses: string[] = [];
  for (const url of new Set(urls)) {
    const hit = cache.get(url);
    if (hit) results.set(url, hit);
    else misses.push(url);
  }
  for (const [url, r] of await checkUrls(misses, opts)) {
    cache.set(r);
    results.set(url, r);
  }
  return results;
}
