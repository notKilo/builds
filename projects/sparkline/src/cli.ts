#!/usr/bin/env node
import { parseArgs } from "node:util";
import { Store } from "./store.ts";
import { createServer } from "./server.ts";
import { autosave, loadSnapshot } from "./persist.ts";

const USAGE = `usage:
  sparkline serve [--port 8080] [--host 127.0.0.1] [--snapshot FILE] [--interval SECONDS]
  sparkline push <name> <value> [--url http://127.0.0.1:8080]`;

export async function serve(args: string[]): Promise<void> {
  const { values } = parseArgs({
    args,
    options: {
      port: { type: "string", default: "8080" },
      host: { type: "string", default: "127.0.0.1" },
      snapshot: { type: "string" },
      interval: { type: "string", default: "30" },
    },
  });
  const store = new Store();
  let stopAutosave = () => {};
  if (values.snapshot) {
    if (loadSnapshot(store, values.snapshot)) console.log(`restored ${store.names().length} series from ${values.snapshot}`);
    stopAutosave = autosave(store, values.snapshot, Number(values.interval) * 1000);
  }
  const server = createServer(store);
  server.listen(Number(values.port), values.host, () => {
    console.log(`listening on http://${values.host}:${values.port}`);
  });
  const shutdown = () => {
    stopAutosave();
    server.close(() => process.exit(0));
    server.closeAllConnections();
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);
}

export async function push(args: string[]): Promise<number> {
  const { values, positionals } = parseArgs({
    args,
    allowPositionals: true,
    options: { url: { type: "string", default: "http://127.0.0.1:8080" } },
  });
  const [name, raw] = positionals;
  const value = Number(raw);
  if (!name || raw === undefined || Number.isNaN(value)) {
    console.error(USAGE);
    return 2;
  }
  const res = await fetch(new URL("/ingest", values.url), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ name, value }),
  });
  if (!res.ok) {
    console.error(`push failed: ${res.status} ${await res.text()}`);
    return 1;
  }
  return 0;
}

async function main(argv: string[]): Promise<void> {
  const [cmd, ...rest] = argv;
  if (cmd === "serve") return serve(rest);
  if (cmd === "push") {
    process.exitCode = await push(rest);
    return;
  }
  console.error(USAGE);
  process.exitCode = cmd === "--help" || cmd === "-h" ? 0 : 2;
}

main(process.argv.slice(2)).catch((e) => {
  console.error((e as Error).message);
  process.exit(1);
});
