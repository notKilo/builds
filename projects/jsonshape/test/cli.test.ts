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

import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

function twoFiles(a: unknown, b: unknown): [string, string] {
  const dir = mkdtempSync(join(tmpdir(), "jsonshape-"));
  writeFileSync(join(dir, "a.json"), JSON.stringify(a));
  writeFileSync(join(dir, "b.json"), JSON.stringify(b));
  return [join(dir, "a.json"), join(dir, "b.json")];
}

test("cli diff prints changes and exits 0 by default", () => {
  const [a, b] = twoFiles({ id: 1, name: "x" }, { id: 1 });
  const r = run(["diff", a, b]);
  assert.equal(r.status, 0);
  assert.match(r.stdout, /BREAKING\s+\$\.name/);
});

test("cli diff --fail-on-breaking exits 3, --json is parseable", () => {
  const [a, b] = twoFiles({ id: 1, name: "x" }, { id: 1 });
  assert.equal(run(["diff", "--fail-on-breaking", a, b]).status, 3);
  const changes = JSON.parse(run(["diff", "--json", a, b]).stdout);
  assert.equal(changes[0].kind, "prop-removed");
  const [c, d] = twoFiles({ id: 1 }, { id: 1, extra: true });
  assert.equal(run(["diff", "--fail-on-breaking", c, d]).status, 0);
});

test("cli diff requires two files", () => {
  assert.equal(run(["diff", "only-one.json"]).status, 2);
});
