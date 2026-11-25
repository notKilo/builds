#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { parseDeck } from "./deck.ts";
import { renderDeckHtml } from "./template.ts";
import { startServer } from "./server.ts";

const USAGE = `usage:
  slidedown build <deck.md> [-o out.html]
  slidedown serve <deck.md> [-p port]
`;

async function main(argv: string[]): Promise<number> {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: { out: { type: "string", short: "o" }, port: { type: "string", short: "p", default: "3000" } },
  });
  const [cmd, file] = positionals;
  if (!file || (cmd !== "build" && cmd !== "serve")) {
    process.stderr.write(USAGE);
    return 2;
  }
  try {
    readFileSync(file);
  } catch {
    process.stderr.write(`cannot read ${file}\n`);
    return 1;
  }
  if (cmd === "build") {
    const out = values.out ?? file.replace(/\.md$/i, "") + ".html";
    writeFileSync(out, renderDeckHtml(parseDeck(readFileSync(file, "utf8"))));
    process.stdout.write(`wrote ${out}\n`);
    return 0;
  }
  const dev = await startServer(file, Number(values.port));
  process.stdout.write(`serving ${file} at http://127.0.0.1:${dev.port}/ (live reload on)\n`);
  return new Promise(() => {});
}

main(process.argv.slice(2)).then(
  (code) => {
    if (code !== 0) process.exit(code);
  },
  (err) => {
    process.stderr.write(`${err instanceof Error ? err.message : err}\n`);
    process.exit(1);
  },
);
