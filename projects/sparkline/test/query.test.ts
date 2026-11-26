import { test } from "node:test";
import assert from "node:assert/strict";
import { aggregate, parseWindow, percentile } from "../src/query.ts";
import type { Point } from "../src/store.ts";

const pts: Point[] = [
  { t: 0, count: 2, sum: 4, min: 1, max: 3 },
  { t: 1000, count: 1, sum: 10, min: 10, max: 10 },
];

test("aggregate computes each aggregation", () => {
  assert.equal(aggregate(pts, "min"), 1);
  assert.equal(aggregate(pts, "max"), 10);
  assert.equal(aggregate(pts, "sum"), 14);
  assert.equal(aggregate(pts, "count"), 3);
  assert.equal(aggregate(pts, "avg"), 14 / 3);
  assert.equal(aggregate(pts, "p95"), 10);
  assert.equal(aggregate([], "avg"), null);
});

test("percentile uses nearest rank", () => {
  assert.equal(percentile([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 50), 5);
  assert.equal(percentile([7], 95), 7);
});

test("parseWindow understands units", () => {
  assert.equal(parseWindow("30s"), 30_000);
  assert.equal(parseWindow("5m"), 300_000);
  assert.equal(parseWindow("1500"), 1500);
  assert.throws(() => parseWindow("abc"));
  assert.throws(() => parseWindow("0s"));
});
