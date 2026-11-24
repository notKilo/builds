import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { UrlCache, checkUrlsCached } from "../src/cache.ts";
import { defaultHttpOptions } from "../src/http.ts";

test("entries expire after the TTL", () => {
  let t = 1000;
  const cache = new UrlCache(null, 500, () => t);
  cache.set({ url: "http://a", ok: true, status: 200 });
  assert.equal(cache.get("http://a")?.status, 200);
  t = 1499;
  assert.ok(cache.get("http://a"));
  t = 1500;
  assert.equal(cache.get("http://a"), undefined);
});

test("persists to disk and reloads; corrupt file is ignored", async () => {
  const dir = await mkdtemp(path.join(tmpdir(), "mdlc-"));
  const file = path.join(dir, "cache.json");
  const a = new UrlCache(file, 60_000);
  a.set({ url: "http://a", ok: false, error: "timed out" });
  await a.save();
  const b = new UrlCache(file, 60_000);
  await b.load();
  assert.deepEqual(b.get("http://a"), { url: "http://a", ok: false, status: undefined, error: "timed out" });

  await import("node:fs/promises").then((fs) => fs.writeFile(file, "{not json"));
  const c = new UrlCache(file, 60_000);
  await c.load();
  assert.equal(c.get("http://a"), undefined);
  assert.match(await readFile(file, "utf8"), /not json/); // load never rewrites
});

test("checkUrlsCached only fetches misses", async () => {
  const calls: string[] = [];
  const fetchImpl = (async (url: string) => {
    calls.push(url);
    return new Response(null, { status: 200 });
  }) as unknown as typeof fetch;
  const opts = { ...defaultHttpOptions, fetchImpl };
  const cache = new UrlCache(null, 60_000);
  cache.set({ url: "http://cached", ok: true, status: 200 });
  const res = await checkUrlsCached(["http://cached", "http://new", "http://new"], opts, cache);
  assert.deepEqual(calls, ["http://new"]);
  assert.equal(res.size, 2);
  assert.ok(cache.get("http://new"));
});
