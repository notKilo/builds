import { test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
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
