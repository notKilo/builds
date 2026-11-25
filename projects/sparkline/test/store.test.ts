import { test } from "node:test";
import assert from "node:assert/strict";
import { Store } from "../src/store.ts";

test("aggregates points in the same bucket", () => {
  const s = new Store({ resolutionMs: 1000 });
  s.record("a", 2, 1000);
  s.record("a", 6, 1999);
  const [p] = s.points("a");
  assert.deepEqual(p, { t: 1000, count: 2, sum: 8, min: 2, max: 6 });
});

test("keeps buckets sorted when points arrive late", () => {
  const s = new Store({ resolutionMs: 1000 });
  s.record("a", 1, 3000);
  s.record("a", 1, 1000);
  s.record("a", 1, 2000);
  assert.deepEqual(s.points("a").map((p) => p.t), [1000, 2000, 3000]);
});

test("drops oldest buckets beyond capacity", () => {
  const s = new Store({ resolutionMs: 1000, capacity: 3 });
  for (let i = 0; i < 5; i++) s.record("a", i, i * 1000);
  assert.deepEqual(s.points("a").map((p) => p.sum), [2, 3, 4]);
});

test("filters by time window and rejects non-finite values", () => {
  const s = new Store({ resolutionMs: 1000 });
  for (let i = 0; i < 5; i++) s.record("a", i, i * 1000);
  assert.equal(s.points("a", 1000, 2000).length, 2);
  assert.throws(() => s.record("a", NaN, 0), RangeError);
  assert.deepEqual(s.names(), ["a"]);
});
