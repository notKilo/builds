#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { inferAll } from "./shape.ts";
import { parseSamples } from "./input.ts";
import { emitTypeScript } from "./emit-ts.ts";
import { emitJsonSchema } from "./emit-schema.ts";
import { diffShapes, formatChanges } from "./diff.ts";
import type { Shape } from "./shape.ts";

const USAGE = `Usage: jsonshape [options] [file...]
       jsonshape diff [options] <old> <new>

Infer a schema from JSON samples. Reads stdin when no files are given.
Each file may hold one JSON document or NDJSON; a top-level array in a
file is treated as a single sample. Use --each to treat its elements
as separate samples instead.

Options:
  -f, --format <ts|schema>  Output format (default: ts)
  -n, --name <name>         Root type name / schema title (default: Root)
      --interface           Emit an interface for the root object (ts only)
      --each                Treat elements of top-level arrays as samples
  -h, --help                Show this help

Diff: compare the shapes inferred from two files and list changes, marking
those that could break consumers of the old shape.
  --json                    Print changes as JSON
  --fail-on-breaking        Exit with status 3 when breaking changes exist
`;

function loadShape(sources: string[], each: boolean): Shape | null {
  let samples: unknown[] = [];
  for (const src of sources) {
    const text = src === "-" ? readFileSync(0, "utf8") : readFileSync(src, "utf8");
    samples.push(...parseSamples(text, src === "-" ? "stdin" : src));
  }
  if (each) samples = samples.flatMap((s) => (Array.isArray(s) ? s : [s]));
  return inferAll(samples);
}

function runDiff(argv: string[]): number {
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: {
      each: { type: "boolean", default: false },
      json: { type: "boolean", default: false },
      "fail-on-breaking": { type: "boolean", default: false },
    },
  });
  if (positionals.length !== 2) {
    process.stderr.write("jsonshape: diff needs exactly two files: <old> <new>\n");
    return 2;
  }
  const changes = diffShapes(
    loadShape([positionals[0]!], values.each),
    loadShape([positionals[1]!], values.each),
  );
  process.stdout.write(values.json ? JSON.stringify(changes, null, 2) + "\n" : formatChanges(changes));
  return values["fail-on-breaking"] && changes.some((c) => c.breaking) ? 3 : 0;
}

function main(argv: string[]): number {
  if (argv[0] === "diff") return runDiff(argv.slice(1));
  const { values, positionals } = parseArgs({
    args: argv,
    allowPositionals: true,
    options: {
      format: { type: "string", short: "f", default: "ts" },
      name: { type: "string", short: "n", default: "Root" },
      interface: { type: "boolean", default: false },
      each: { type: "boolean", default: false },
      help: { type: "boolean", short: "h", default: false },
    },
  });
  if (values.help) {
    process.stdout.write(USAGE);
    return 0;
  }
  if (values.format !== "ts" && values.format !== "schema") {
    process.stderr.write(`jsonshape: unknown format "${values.format}"\n`);
    return 2;
  }

  const shape = loadShape(positionals.length ? positionals : ["-"], values.each);
  process.stdout.write(
    values.format === "ts"
      ? emitTypeScript(shape, { name: values.name, interfaces: values.interface })
      : emitJsonSchema(shape, values.name === "Root" ? undefined : values.name),
  );
  return 0;
}

try {
  process.exitCode = main(process.argv.slice(2));
} catch (e) {
  process.stderr.write(`jsonshape: ${(e as Error).message}\n`);
  process.exitCode = 1;
}
