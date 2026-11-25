import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";

const cli = join(import.meta.dirname, "../src/cli.ts");
const tmp = mkdtempSync(join(tmpdir(), "slidedown-pdf-"));
const md = join(tmp, "d.md");
writeFileSync(md, "# One\n---\n# Two");

let hasPlaywright = true;
try {
  await import("playwright" as string);
} catch {
  hasPlaywright = false;
}

test("pdf command exports one page per slide", { skip: !hasPlaywright }, () => {
  const out = join(tmp, "d.pdf");
  execFileSync("node", [cli, "pdf", md, "-o", out], { stdio: "pipe" });
  const pdf = readFileSync(out).toString("latin1");
  assert.ok(pdf.startsWith("%PDF"));
  assert.equal((pdf.match(/\/Type \/Page\b/g) ?? []).length, 2);
});

test("pdf command explains how to get playwright when missing", { skip: hasPlaywright }, () => {
  assert.throws(() => execFileSync("node", [cli, "pdf", md], { stdio: "pipe" }), /npm install playwright/);
});
