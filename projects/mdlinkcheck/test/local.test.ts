import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, mkdir } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { AnchorIndex, checkLocalLink, isExternal } from "../src/local.ts";
import type { Link } from "../src/extract.ts";

async function fixture() {
  const root = await mkdtemp(path.join(tmpdir(), "mdlc-"));
  await mkdir(path.join(root, "docs"));
  await writeFile(path.join(root, "README.md"), "# Title\n## Getting Started\n");
  await writeFile(path.join(root, "docs", "a.md"), "# A\n");
  await writeFile(path.join(root, "docs", "my file.md"), "# Spaced\n");
  return root;
}
const link = (target: string): Link => ({ kind: "inline", target, line: 3, column: 5 });

test("valid relative file, anchor and root-relative links pass", async () => {
  const root = await fixture();
  const idx = new AnchorIndex();
  const from = path.join(root, "docs", "a.md");
  for (const t of ["../README.md", "../README.md#getting-started", "#a", "/README.md", "my%20file.md", "."]) {
    assert.equal(await checkLocalLink(from, link(t), root, idx), null, t);
  }
});

test("missing file and missing anchor are reported with position", async () => {
  const root = await fixture();
  const idx = new AnchorIndex();
  const from = path.join(root, "docs", "a.md");
  const f = await checkLocalLink(from, link("nope.md"), root, idx);
  assert.equal(f?.rule, "missing-file");
  assert.equal(f?.line, 3);
  const a = await checkLocalLink(from, link("../README.md#install"), root, idx);
  assert.equal(a?.rule, "missing-anchor");
  const self = await checkLocalLink(from, link("#nope"), root, idx);
  assert.equal(self?.rule, "missing-anchor");
});

test("external schemes are skipped", async () => {
  assert.ok(isExternal("https://x.test") && isExternal("mailto:a@b.c") && isExternal("//cdn.x"));
  assert.ok(!isExternal("docs/a.md"));
});
