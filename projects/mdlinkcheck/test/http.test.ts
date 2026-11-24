import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import http from "node:http";
import type { AddressInfo } from "node:net";
import { checkUrl, checkUrls, defaultHttpOptions } from "../src/http.ts";

let server: http.Server;
let base: string;
const hits: string[] = [];

before(async () => {
  server = http.createServer((req, res) => {
    hits.push(`${req.method} ${req.url}`);
    if (req.url === "/ok") return void res.end("hi");
    if (req.url === "/gone") return void res.writeHead(404).end();
    if (req.url === "/nohead") return void res.writeHead(req.method === "HEAD" ? 405 : 200).end();
    if (req.url === "/redirect") return void res.writeHead(302, { location: "/ok" }).end();
    if (req.url === "/slow") return; // never answers
    res.writeHead(500).end();
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
});
after(() => { server.closeAllConnections(); server.close(); });

const opts = { ...defaultHttpOptions, timeoutMs: 300 };

test("2xx and redirects are ok, 404 is not", async () => {
  assert.equal((await checkUrl(`${base}/ok`, opts)).ok, true);
  assert.equal((await checkUrl(`${base}/redirect`, opts)).ok, true);
  const gone = await checkUrl(`${base}/gone`, opts);
  assert.deepEqual([gone.ok, gone.status], [false, 404]);
});

test("falls back to GET when HEAD is rejected", async () => {
  const r = await checkUrl(`${base}/nohead`, opts);
  assert.equal(r.ok, true);
  assert.ok(hits.includes("GET /nohead"));
});

test("timeouts and connection errors are reported", async () => {
  assert.equal((await checkUrl(`${base}/slow`, opts)).error, "timed out");
  const refused = await checkUrl("http://127.0.0.1:1/x", opts);
  assert.equal(refused.ok, false);
  assert.ok(refused.error);
});

test("checkUrls dedupes and bounds work", async () => {
  hits.length = 0;
  const res = await checkUrls([`${base}/ok`, `${base}/ok`, `${base}/gone`], { ...opts, concurrency: 2 });
  assert.equal(res.size, 2);
  assert.equal(hits.filter((h) => h === "HEAD /ok").length, 1);
});
