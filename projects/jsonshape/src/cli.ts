#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { parseArgs } from "node:util";
import { inferAll } from "./shape.ts";
import { parseSamples } from "./input.ts";
import { emitTypeScript } from "./emit-ts.ts";
import { emitJsonSchema } from "./emit-schema.ts";

const USAGE = `Usage: jsonshape [options] [file...]

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
`;

function main(argv: string[]): number {
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

  let samples: unknown[] = [];
  const sources = positionals.length ? positionals : ["-"];
  for (const src of sources) {
    const text = src === "-" ? readFileSync(0, "utf8") : readFileSync(src, "utf8");
    samples.push(...parseSamples(text, src === "-" ? "stdin" : src));
  }
  if (values.each) samples = samples.flatMap((s) => (Array.isArray(s) ? s : [s]));

  const shape = inferAll(samples);
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
