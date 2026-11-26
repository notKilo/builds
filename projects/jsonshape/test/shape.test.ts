import { test } from "node:test";
import assert from "node:assert/strict";
import { inferShape, inferAll, detectFormat, enumValues, type Shape } from "../src/shape.ts";

test("primitives", () => {
  assert.deepEqual(inferShape(null), { kind: "null" });
  assert.deepEqual(inferShape(true), { kind: "boolean" });
  assert.deepEqual(inferShape(3), { kind: "number", integer: true });
  assert.deepEqual(inferShape(3.5), { kind: "number", integer: false });
});

test("integers merged with floats become non-integer", () => {
  assert.deepEqual(inferAll([1, 2.5]), { kind: "number", integer: false });
});

test("mixed kinds form a union", () => {
  const s = inferAll([1, "a", null]) as Extract<Shape, { kind: "union" }>;
  assert.equal(s.kind, "union");
  assert.deepEqual(s.options.map((o) => o.kind), ["number", "string", "null"]);
});

test("object keys missing from some samples have lower seen counts", () => {
  const s = inferAll([{ a: 1, b: 2 }, { a: 3 }]) as Extract<Shape, { kind: "object" }>;
  assert.equal(s.count, 2);
  assert.equal(s.props.a!.seen, 2);
  assert.equal(s.props.b!.seen, 1);
});

test("arrays merge element shapes; empty arrays have no item shape", () => {
  assert.deepEqual(inferShape([]), { kind: "array", items: null });
  const s = inferShape([{ x: 1 }, { y: "a" }]) as Extract<Shape, { kind: "array" }>;
  assert.deepEqual(Object.keys((s.items as Extract<Shape, { kind: "object" }>).props), ["x", "y"]);
});

test("empty array merged with populated array keeps items", () => {
  const s = inferAll([[], [1]]) as Extract<Shape, { kind: "array" }>;
  assert.deepEqual(s.items, { kind: "number", integer: true });
});

test("string formats", () => {
  assert.equal(detectFormat("2024-01-02"), "date");
  assert.equal(detectFormat("2024-01-02T03:04:05Z"), "date-time");
  assert.equal(detectFormat("123e4567-e89b-12d3-a456-426614174000"), "uuid");
  assert.equal(detectFormat("a@b.co"), "email");
  assert.equal(detectFormat("https://x.test/a"), "uri");
  assert.equal(detectFormat("hello"), undefined);
});

test("differing formats drop the format", () => {
  const s = inferAll(["2024-01-02", "hello"]) as Extract<Shape, { kind: "string" }>;
  assert.equal(s.format, undefined);
});

test("enum inferred only with repeats and few distinct values", () => {
  const e = inferAll(["a", "b", "a", "b"]) as Extract<Shape, { kind: "string" }>;
  assert.deepEqual(enumValues(e), ["a", "b"]);
  const unique = inferAll(["a", "b", "c"]) as Extract<Shape, { kind: "string" }>;
  assert.equal(enumValues(unique), undefined);
  const many = inferAll(Array.from({ length: 20 }, (_, i) => `v${i % 10}`)) as Extract<Shape, { kind: "string" }>;
  assert.equal(enumValues(many), undefined);
});
