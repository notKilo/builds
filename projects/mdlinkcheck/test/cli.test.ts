import { test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import os from "node:os";
import { mkdtemp, writeFile } from "node:fs/promises";
import { main } from "../src/cli.ts";

const site = path.join(import.meta.dirname, "fixtures", "site");

/** Run the CLI in-process, capturing its output. */
async function cli(args: string[]) {
  let out = "";
  let err = "";
  const code = await main(args, { out: (s) => void (out += s), err: (s) => void (err += s) });
  return { code, out, err };
}

test("reports broken links and exits 1", async () => {
  const r = await cli([site, "--root", site]);
  assert.equal(r.code, 1);
  assert.match(r.out, /README\.md:3:\d+ error \[missing-file\]/);
  assert.match(r.out, /\[missing-anchor\]/);
  assert.match(r.out, /4 links in 2 files checked: 2 error/);
});

test("clean directory exits 0", async () => {
  const r = await cli([path.join(site, "docs"), "--root", site]);
  assert.equal(r.code, 0);
});

test("json format is parseable", async () => {
  const r = await cli([site, "--root", site, "-f", "json"]);
  assert.equal(JSON.parse(r.out).problems.length, 2);
});

test("usage errors exit 2", async () => {
  assert.equal((await cli([])).code, 2);
  assert.equal((await cli(["x", "--format", "xml"])).code, 2);
  assert.equal((await cli(["x", "--bogus"])).code, 2);
  assert.equal((await cli(["x", "--timeout", "-5"])).code, 2);
});

test("--help exits 0", async () => {
  const r = await cli(["--help"]);
  assert.equal(r.code, 0);
  assert.match(r.out, /Usage: mdlinkcheck/);
});

test("--ignore-path skips matching files", async () => {
  const r = await cli([site, "--root", site, "--ignore-path", "README.md", "-f", "json"]);
  assert.equal(JSON.parse(r.out).filesChecked, 1);
});

test("config file supplies ignores and defaults; flags win", async () => {
  const dir = await mkdtemp(path.join(os.tmpdir(), "mdlc-cli-"));
  const cfg = path.join(dir, "cfg.json");
  await writeFile(cfg, JSON.stringify({ ignorePaths: ["README.md"], format: "json" }));
  const fromConfig = await cli([site, "--root", site, "--config", cfg]);
  assert.equal(JSON.parse(fromConfig.out).filesChecked, 1);
  const overridden = await cli([site, "--root", site, "--config", cfg, "-f", "text"]);
  assert.match(overridden.out, /links in 1 files? checked/);
});

test("ignored URL prefixes are not checked", async () => {
  const dir = await mkdtemp(path.join(os.tmpdir(), "mdlc-cli-"));
  await writeFile(path.join(dir, "a.md"), "[x](http://localhost:9/nope) [y](http://127.0.0.1:9/z)\n");
  const r = await cli([dir, "--root", dir, "-e", "--no-cache", "--timeout", "500", "--ignore-url", "http://localhost", "-f", "json"]);
  const targets = JSON.parse(r.out).problems.map((p: { target: string }) => p.target);
  assert.deepEqual(targets, ["http://127.0.0.1:9/z"]);
});

test("bad config exits 2", async () => {
  const dir = await mkdtemp(path.join(os.tmpdir(), "mdlc-cli-"));
  const cfg = path.join(dir, "cfg.json");
  await writeFile(cfg, `{"bogus": 1}`);
  const r = await cli([site, "--config", cfg]);
  assert.equal(r.code, 2);
  assert.match(r.err, /unknown config key/);
});
