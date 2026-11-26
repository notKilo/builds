import { test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Store } from "../src/store.ts";
import { loadSnapshot, saveSnapshot } from "../src/persist.ts";

function tmpFile() {
  return path.join(fs.mkdtempSync(path.join(os.tmpdir(), "spark-")), "snap.json");
}

test("snapshot round-trips series", () => {
  const a = new Store({ resolutionMs: 1000 });
  a.record("cpu", 1, 1000);
  a.record("cpu", 3, 1500);
  a.record("mem", 7, 2000);
  const file = tmpFile();
  saveSnapshot(a, file);
  const b = new Store({ resolutionMs: 1000 });
  assert.equal(loadSnapshot(b, file), true);
  assert.deepEqual(b.names(), ["cpu", "mem"]);
  assert.deepEqual(b.points("cpu"), a.points("cpu"));
  assert.equal(fs.existsSync(`${file}.tmp`), false);
});

test("loadSnapshot returns false when file is missing", () => {
  assert.equal(loadSnapshot(new Store(), tmpFile()), false);
});

test("restore trims to capacity and rejects unknown versions", () => {
  const a = new Store({ capacity: 10 });
  for (let i = 0; i < 10; i++) a.record("x", i, i * 1000);
  const small = new Store({ capacity: 3 });
  small.restore(a.snapshot());
  assert.equal(small.points("x").length, 3);
  assert.equal(small.points("x")[0].t, 7000);
  assert.throws(() => small.restore({ ...a.snapshot(), version: 2 as 1 }));
});
