import { test } from "node:test";
import assert from "node:assert/strict";
import type { AddressInfo } from "node:net";
import { Store } from "../src/store.ts";
import { createServer, parseBody } from "../src/server.ts";

async function withServer(fn: (base: string, store: Store) => Promise<void>) {
  const store = new Store({ resolutionMs: 1000 });
  const server = createServer(store, () => 10_000);
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  try {
    await fn(`http://127.0.0.1:${(server.address() as AddressInfo).port}`, store);
  } finally {
    server.close();
  }
}

test("parseBody handles line protocol and JSON", () => {
  assert.deepEqual(parseBody("cpu:1.5\nweb.hits:3\n", "text/plain"), [
    { name: "cpu", value: 1.5 },
    { name: "web.hits", value: 3 },
  ]);
  assert.deepEqual(parseBody('{"name":"a","value":2,"at":5}', "application/json"), [{ name: "a", value: 2, at: 5 }]);
  assert.throws(() => parseBody("nonsense", "text/plain"));
  assert.throws(() => parseBody("[{\"name\":1}]", "application/json"));
});

test("ingest then fetch sparkline and series list", async () => {
  await withServer(async (base) => {
    const r = await fetch(`${base}/ingest`, { method: "POST", body: "cpu:1\ncpu:3\nmem:5" });
    assert.equal(r.status, 202);
    assert.deepEqual(await r.json(), { accepted: 3 });
    assert.deepEqual(await (await fetch(`${base}/api/series`)).json(), ["cpu", "mem"]);
    const svg = await fetch(`${base}/spark/cpu.svg?style=bars&w=100&color=ff0000`);
    assert.equal(svg.headers.get("content-type"), "image/svg+xml");
    const text = await svg.text();
    assert.match(text, /width="100"/);
    assert.match(text, /#ff0000/);
  });
});

test("rejects bad input and unknown series", async () => {
  await withServer(async (base) => {
    assert.equal((await fetch(`${base}/ingest`, { method: "POST", body: "bad name!:1" })).status, 400);
    assert.equal((await fetch(`${base}/spark/nope.svg`)).status, 404);
    assert.equal((await fetch(`${base}/other`)).status, 404);
  });
});

test("window parameter drops old points", async () => {
  await withServer(async (base, store) => {
    store.record("a", 1, 1000);
    store.record("a", 9, 9000);
    const text = await (await fetch(`${base}/spark/a.svg?window=2000&markers=0`)).text();
    assert.match(text, /<circle/); // single remaining point
  });
});

test("series query endpoint aggregates within a window", async () => {
  await withServer(async (base) => {
    await fetch(`${base}/ingest`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify([
        { name: "lat", value: 100, at: 1000 },
        { name: "lat", value: 10, at: 8000 },
        { name: "lat", value: 30, at: 9000 },
      ]),
    });
    const all = await (await fetch(`${base}/api/series/lat?agg=max`)).json();
    assert.equal(all.value, 100);
    const recent = await (await fetch(`${base}/api/series/lat?agg=avg&window=5s`)).json();
    assert.equal(recent.value, 20);
    assert.equal(recent.buckets, 2);
    assert.equal((await fetch(`${base}/api/series/lat?agg=nope`)).status, 400);
    assert.equal((await fetch(`${base}/api/series/missing`)).status, 404);
  });
});
