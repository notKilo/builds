import { test } from "node:test";
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const CLI = path.resolve("src/cli.ts");

function startServer(args: string[]) {
  const child = spawn(process.execPath, [CLI, "serve", ...args], { stdio: ["ignore", "pipe", "inherit"] });
  const ready = new Promise<void>((resolve, reject) => {
    let out = "";
    child.stdout.on("data", (d) => {
      out += d;
      if (out.includes("listening on")) resolve();
    });
    child.on("exit", () => reject(new Error("server exited early")));
  });
  return { child, ready };
}

test("serve, push, snapshot on SIGTERM and restore", async () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "sparkline-"));
  const snap = path.join(dir, "snap.json");
  const port = 18000 + Math.floor(Math.random() * 1000);
  const url = `http://127.0.0.1:${port}`;

  const a = startServer(["--port", String(port), "--snapshot", snap]);
  await a.ready;
  const r = spawnSync(process.execPath, [CLI, "push", "cpu", "42", "--url", url]);
  assert.equal(r.status, 0, r.stderr.toString());
  assert.deepEqual(await (await fetch(`${url}/api/series`)).json(), ["cpu"]);
  const exited = new Promise((res) => a.child.once("exit", res));
  a.child.kill("SIGTERM");
  await exited;
  assert.ok(fs.existsSync(snap), "snapshot written on shutdown");

  const b = startServer(["--port", String(port), "--snapshot", snap]);
  await b.ready;
  try {
    assert.deepEqual(await (await fetch(`${url}/api/series`)).json(), ["cpu"]);
  } finally {
    b.child.kill("SIGTERM");
  }
});

test("push validates arguments", () => {
  const r = spawnSync(process.execPath, [CLI, "push", "cpu", "abc"]);
  assert.equal(r.status, 2);
  assert.match(r.stderr.toString(), /usage/);
});
