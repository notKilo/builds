import { test } from "node:test";
import assert from "node:assert/strict";
import { inferAll } from "../src/shape.ts";
import { diffShapes, formatChanges } from "../src/diff.ts";

const diff = (a: unknown[], b: unknown[]) => diffShapes(inferAll(a), inferAll(b));
const find = (cs: ReturnType<typeof diff>, kind: string) => cs.find((c) => c.kind === kind);

test("identical shapes have no changes", () => {
  assert.deepEqual(diff([{ a: 1 }], [{ a: 2 }]), []);
});

test("removed required prop is breaking, optional is not", () => {
  const cs = diff([{ a: 1, b: 2 }, { a: 1 }], [{ a: 1 }]);
  const c = find(cs, "prop-removed")!;
  assert.equal(c.path, "$.b");
  assert.equal(c.breaking, false);
  const cs2 = diff([{ a: 1, b: 2 }], [{ a: 1 }]);
  assert.equal(find(cs2, "prop-removed")!.breaking, true);
});

test("added props are non-breaking", () => {
  const c = find(diff([{ a: 1 }], [{ a: 1, b: "x" }]), "prop-added")!;
  assert.equal(c.breaking, false);
  assert.equal(c.path, "$.b");
});

test("required to optional is breaking, the reverse is not", () => {
  const cs = diff([{ a: 1 }, { a: 2 }], [{ a: 1 }, {}]);
  assert.equal(find(cs, "became-optional")!.breaking, true);
  const back = diff([{ a: 1 }, {}], [{ a: 1 }, { a: 2 }]);
  assert.equal(find(back, "became-required")!.breaking, false);
});

test("type changes: widening breaks, narrowing does not", () => {
  const widen = diff([{ a: 1 }], [{ a: 1 }, { a: "x" }]);
  const c = find(widen, "type-added")!;
  assert.equal(c.breaking, true);
  assert.equal(c.path, "$.a");
  const narrow = diff([{ a: 1 }, { a: "x" }], [{ a: 1 }]);
  assert.equal(find(narrow, "type-removed")!.breaking, false);
});

test("integer to float is breaking", () => {
  assert.equal(find(diff([{ n: 1 }], [{ n: 1.5 }]), "integer-widened")!.breaking, true);
  assert.equal(find(diff([{ n: 1.5 }], [{ n: 1 }]), "integer-narrowed")!.breaking, false);
});

test("recurses into arrays and nested objects", () => {
  const cs = diff([{ items: [{ id: 1, tag: "a" }] }], [{ items: [{ id: 1 }] }]);
  assert.equal(find(cs, "prop-removed")!.path, "$.items[].tag");
});

test("enum values added is breaking, removed is not", () => {
  const mk = (...v: string[]) => v.map((s) => ({ s }));
  const old = mk("a", "b", "a", "b");
  const added = diff(old, mk("a", "b", "c", "a", "b", "c"));
  assert.equal(find(added, "enum-changed")!.breaking, true);
  const removed = diff(mk("a", "b", "c", "a", "b", "c"), old);
  assert.equal(find(removed, "enum-changed")!.breaking, false);
});

test("format change is reported", () => {
  const c = find(diff([{ d: "2024-01-01" }], [{ d: "hello" }]), "format-changed")!;
  assert.match(c.message, /date -> none/);
});

test("formatChanges lists breaking first with a summary", () => {
  const out = formatChanges(diff([{ a: 1 }], [{ b: 1 }]));
  assert.ok(out.indexOf("BREAKING") < out.indexOf("non-breaking"));
  assert.match(out, /1 breaking, 1 non-breaking/);
  assert.equal(formatChanges([]), "No differences.\n");
});
