import { test } from "node:test";
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { execFileSync } from "node:child_process";
import { startServer } from "../src/server.ts";

const tmp = (): string => mkdtempSync(join(tmpdir(), "slidedown-"));

test("serves the rendered deck with live reload client", async () => {
  const file = join(tmp(), "d.md");
  writeFileSync(file, "# Hello\n---\n# Two");
  const dev = await startServer(file, 0);
  try {
    const res = await fetch(`http://127.0.0.1:${dev.port}/`);
    const html = await res.text();
    assert.equal(res.status, 200);
    assert.match(html, /<h1 id="hello">Hello<\/h1>/);
    assert.match(html, /__reload/);
    assert.equal((await fetch(`http://127.0.0.1:${dev.port}/nope`)).status, 404);
  } finally {
    dev.close();
  }
});

test("pushes an SSE event when the file changes", async () => {
  const file = join(tmp(), "d.md");
  writeFileSync(file, "# A");
  const dev = await startServer(file, 0);
  const ac = new AbortController();
  try {
    const res = await fetch(`http://127.0.0.1:${dev.port}/__reload`, { signal: ac.signal });
    const reader = res.body!.getReader();
    await reader.read(); // ": connected"
    setTimeout(() => writeFileSync(file, "# B"), 50);
    const chunk = await reader.read();
    assert.match(new TextDecoder().decode(chunk.value), /data: reload/);
  } finally {
    ac.abort();
    dev.close();
  }
});

test("build command writes an html file", () => {
  const dir = tmp();
  const file = join(dir, "talk.md");
  writeFileSync(file, "# Hi");
  const out = join(dir, "out.html");
  execFileSync("node", [join(import.meta.dirname, "../src/cli.ts"), "build", file, "-o", out]);
  assert.match(execFileSync("cat", [out]).toString(), /<h1 id="hi">Hi<\/h1>/);
});

test("cli prints usage on bad input", () => {
  assert.throws(() => execFileSync("node", [join(import.meta.dirname, "../src/cli.ts")], { stdio: "pipe" }), /usage/);
});
