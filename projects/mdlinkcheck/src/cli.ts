#!/usr/bin/env node
import { parseArgs } from "node:util";
import path from "node:path";
import { UrlCache } from "./cache.ts";
import { defaultHttpOptions } from "./http.ts";
import { format, type Format } from "./report.ts";
import { run } from "./run.ts";

const HELP = `Usage: mdlinkcheck [options] <file|dir>...

Checks relative links and #anchors in Markdown files; optionally external URLs.

Options:
  -f, --format <text|json|sarif>  output format (default: text)
  -e, --external                  also check http(s) URLs
      --timeout <ms>              per-request timeout (default: 10000)
      --concurrency <n>           parallel requests (default: 8)
      --cache-ttl <hours>         reuse URL results this long (default: 24)
      --no-cache                  do not read or write the cache
      --cache-file <path>         cache location (default: .mdlinkcheck-cache.json)
      --root <dir>                base for /absolute links (default: cwd)
      --strict                    exit 1 on warnings too
      --ci                        shorthand for --external --strict --no-cache
  -h, --help

Exit codes: 0 clean, 1 problems found, 2 usage error.
`;

export interface Io {
  out: (s: string) => void;
  err: (s: string) => void;
}

const stdio: Io = { out: (s) => void process.stdout.write(s), err: (s) => void process.stderr.write(s) };

export async function main(argv: string[], io: Io = stdio): Promise<number> {
  let parsed;
  try {
    parsed = parseArgs({
      args: argv,
      allowPositionals: true,
      options: {
        format: { type: "string", short: "f", default: "text" },
        external: { type: "boolean", short: "e", default: false },
        timeout: { type: "string" },
        concurrency: { type: "string" },
        "cache-ttl": { type: "string" },
        "no-cache": { type: "boolean", default: false },
        "cache-file": { type: "string", default: ".mdlinkcheck-cache.json" },
        root: { type: "string" },
        strict: { type: "boolean", default: false },
        ci: { type: "boolean", default: false },
        help: { type: "boolean", short: "h", default: false },
      },
    });
  } catch (err) {
    io.err(`${(err as Error).message}\n\n${HELP}`);
    return 2;
  }
  const { values, positionals } = parsed;
  if (values.help) {
    io.out(HELP);
    return 0;
  }
  if (positionals.length === 0) {
    io.err(`No input paths given.\n\n${HELP}`);
    return 2;
  }
  if (!["text", "json", "sarif"].includes(values.format!)) {
    io.err(`Unknown format: ${values.format}\n`);
    return 2;
  }
  const num = (v: string | undefined, fallback: number, name: string): number | null => {
    if (v === undefined) return fallback;
    const n = Number(v);
    if (!Number.isFinite(n) || n <= 0) {
      io.err(`--${name} must be a positive number\n`);
      return null;
    }
    return n;
  };
  const timeoutMs = num(values.timeout, defaultHttpOptions.timeoutMs, "timeout");
  const concurrency = num(values.concurrency, defaultHttpOptions.concurrency, "concurrency");
  const ttlHours = num(values["cache-ttl"], 24, "cache-ttl");
  if (timeoutMs === null || concurrency === null || ttlHours === null) return 2;

  const ci = values.ci!;
  const external = values.external! || ci;
  const strict = values.strict! || ci;
  const noCache = values["no-cache"]! || ci;
  const root = path.resolve(values.root ?? process.cwd());

  const cache = new UrlCache(noCache ? null : path.resolve(values["cache-file"]!), ttlHours * 3_600_000);
  const summary = await run({
    paths: positionals,
    root,
    external,
    http: { ...defaultHttpOptions, timeoutMs, concurrency },
    cache,
  });
  io.out(format(values.format as Format, summary, root));

  const failing = summary.problems.some((p) => p.severity === "error" || strict);
  return failing ? 1 : 0;
}

if (import.meta.main) {
  process.exitCode = await main(process.argv.slice(2));
}
