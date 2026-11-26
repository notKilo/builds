import { test } from "node:test";
import assert from "node:assert/strict";
import { inferAll } from "../src/shape.ts";
import { emitTypeScript } from "../src/emit-ts.ts";
import { emitJsonSchema, toSchema } from "../src/emit-schema.ts";

const samples = [
  { id: 1, name: "a", role: "admin", tags: ["x"], note: null, "odd-key": true },
  { id: 2, name: "b", role: "user", tags: [], extra: 1.5, "odd-key": false },
  { id: 3, name: "c", role: "admin", tags: ["y", "z"], note: "hi", "odd-key": true },
];

test("typescript output marks optional keys, enums, unions and quotes odd keys", () => {
  const ts = emitTypeScript(inferAll(samples), { name: "User", interfaces: true });
  assert.match(ts, /export interface User \{/);
  assert.match(ts, /  id: number;/);
  assert.match(ts, /  role: "admin" \| "user";/);
  assert.match(ts, /  tags: string\[\];/);
  assert.match(ts, /  note\?: null \| string;/);
  assert.match(ts, /  extra\?: number;/);
  assert.match(ts, /  "odd-key": boolean;/);
});

test("nested objects are indented and arrays of unions are wrapped", () => {
  const ts = emitTypeScript(inferAll([{ a: { b: [1, "x"] } }]));
  assert.equal(
    ts,
    "export type Root = {\n  a: {\n    b: Array<number | string>;\n  };\n};\n",
  );
});

test("empty input yields unknown", () => {
  assert.equal(emitTypeScript(null), "export type Root = unknown;\n");
});

test("json schema has required list, integer type and formats", () => {
  const schema = toSchema(inferAll(samples)!) as any;
  assert.equal(schema.type, "object");
  assert.deepEqual(schema.required, ["id", "name", "role", "tags", "odd-key"]);
  assert.equal(schema.properties.id.type, "integer");
  assert.equal(schema.properties.extra.type, "number");
  assert.deepEqual(schema.properties.role.enum, ["admin", "user"]);
  assert.deepEqual(schema.properties.note.anyOf.map((o: any) => o.type), ["null", "string"]);
});

test("json schema document includes dialect and title", () => {
  const doc = JSON.parse(emitJsonSchema(inferAll([{ d: "2024-01-01" }]), "Thing"));
  assert.equal(doc.$schema, "https://json-schema.org/draft/2020-12/schema");
  assert.equal(doc.title, "Thing");
  assert.equal(doc.properties.d.format, "date");
});
