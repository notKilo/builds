import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { loadConfig, parseConfig } from "../src/config.ts";
import { globToRegExp, matchesAny } from "../src/glob.ts";

test("glob: star stays within a segment, globstar crosses", () => {
  assert.ok(globToRegExp("docs/*.md").test("docs/a.md"));
  assert.ok(!globToRegExp("docs/*.md").test("docs/x/a.md"));
  assert.ok(globToRegExp("**/CHANGELOG.md").test("CHANGELOG.md"));
  assert.ok(globToRegExp("**/CHANGELOG.md").test("a/b/CHANGELOG.md"));
  assert.ok(globToRegExp("vendor/**").test("vendor/x/y.md"));
  assert.ok(globToRegExp("a?.md").test("ab.md"));
  assert.ok(!globToRegExp("a.md").test("axmd"));
});

test("matchesAny normalises backslashes", () => {
  assert.ok(matchesAny([globToRegExp("a/*.md")], "a\\b.md"));
});

test("parseConfig accepts valid and rejects invalid shapes", () => {
  assert.deepEqual(parseConfig({ ignoreUrls: ["http://localhost"], strict: true }), { ignoreUrls: ["http://localhost"], strict: true });
  assert.throws(() => parseConfig([]), /JSON object/);
  assert.throws(() => parseConfig({ nope: 1 }), /unknown config key/);
  assert.throws(() => parseConfig({ ignorePaths: "x" }), /array of strings/);
  assert.throws(() => parseConfig({ format: "xml" }), /format/);
  assert.throws(() => parseConfig({ timeout: 0 }), /positive/);
});

test("loadConfig: missing default is empty, missing explicit throws", async () => {
  const dir = await mkdtemp(path.join(os.tmpdir(), "mdlc-"));
  assert.deepEqual(await loadConfig(dir), {});
  await assert.rejects(loadConfig(dir, path.join(dir, "nope.json")), /cannot read config/);
  await writeFile(path.join(dir, ".mdlinkcheck.json"), `{"external": true}`);
  assert.deepEqual(await loadConfig(dir), { external: true });
  await writeFile(path.join(dir, ".mdlinkcheck.json"), `{bad`);
  await assert.rejects(loadConfig(dir), /\.mdlinkcheck\.json/);
});
