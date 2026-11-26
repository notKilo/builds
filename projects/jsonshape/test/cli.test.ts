import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { parseSamples } from "../src/input.ts";

const cli = new URL("../src/cli.ts", import.meta.url).pathname;
const run = (args: string[], input = "") =>
  spawnSync(process.execPath, [cli, ...args], { input, encoding: "utf8" });

test("parseSamples handles documents, NDJSON and empty input", () => {
  assert.deepEqual(parseSamples('{"a":1}'), [{ a: 1 }]);
  assert.deepEqual(parseSamples('{"a":1}\n{"a":2}\n'), [{ a: 1 }, { a: 2 }]);
  assert.deepEqual(parseSamples("  "), []);
});

test("parseSamples reports the failing NDJSON line", () => {
  assert.throws(() => parseSamples('{"a":1}\n{bad}', "f.ndjson"), /f\.ndjson:2/);
});

test("cli reads stdin and emits TypeScript", () => {
  const r = run([], '{"a":1}\n{"a":2,"b":"x"}\n');
  assert.equal(r.status, 0);
  assert.match(r.stdout, /a: number;/);
  assert.match(r.stdout, /b\?: string;/);
});

test("cli --each splits top-level arrays", () => {
  const r = run(["--each", "-f", "schema"], '[{"a":1},{"a":2,"b":true}]');
  const doc = JSON.parse(r.stdout);
  assert.deepEqual(doc.required, ["a"]);
  assert.ok(doc.properties.b);
});

test("cli rejects unknown formats and bad JSON", () => {
  assert.equal(run(["-f", "yaml"]).status, 2);
  const bad = run([], "{nope");
  assert.equal(bad.status, 1);
  assert.match(bad.stderr, /jsonshape:/);
});
