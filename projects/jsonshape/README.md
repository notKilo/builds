# jsonshape

Infer a schema from JSON samples and emit TypeScript types or JSON Schema
(draft 2020-12). It can also diff the shapes of two files and flag changes
that would break consumers. Zero runtime dependencies.

## Install

Requires Node 22.18+ (runs TypeScript directly, no build step).

```sh
npm ci
node src/cli.ts --help
```

## Usage

```sh
jsonshape [options] [file...]        # stdin when no files are given
jsonshape diff [options] <old> <new>
```

Inputs may be a JSON document or NDJSON. Several files are merged into one
shape. A top-level array is one sample unless `--each` is given.

| Option | Meaning |
|---|---|
| `-f, --format <ts\|schema>` | Output format (default `ts`) |
| `-n, --name <name>` | Root type name / schema title |
| `--interface` | Emit `interface` for the root object (ts only) |
| `--each` | Treat elements of top-level arrays as samples |

### What gets inferred

- Keys missing from some samples become optional.
- Differing types become unions; integers and floats are tracked separately.
- Strings matching `date-time`, `date`, `uuid`, `email` or `uri` get a format.
- Small sets of repeated string values (at most 8 distinct, 3+ samples) become enums.

### Example

```sh
$ cat examples/v1.json
{"id":1,"name":"Ada","tags":["a"],"created":"2024-01-02T03:04:05Z"}

$ jsonshape --interface -n User examples/v1.json
export interface User {
  id: number;
  name: string;
  tags: string[];
  created: string;
}

$ jsonshape -f schema -n User examples/v1.json   # JSON Schema, with "format": "date-time"
```

### Diffing

```sh
$ jsonshape diff examples/v1.json examples/v2.json
BREAKING      $.name  required property removed
non-breaking  $.email  required property added

1 breaking, 1 non-breaking
```

Changes are judged from a consumer's view. Breaking: removing a required
property, making a property optional, accepting an additional type, turning
an integer into a float, widening an enum, or changing a string format.
Non-breaking: added properties, optional -> required, narrowed types and enums.

`--json` prints the changes as JSON; `--fail-on-breaking` exits with status 3
if any change is breaking, handy in CI.

## Library

```ts
import { inferAll } from "./src/shape.ts";
import { emitTypeScript } from "./src/emit-ts.ts";
import { diffShapes } from "./src/diff.ts";

const shape = inferAll([{ a: 1 }, { a: 2, b: "x" }]);
emitTypeScript(shape, { name: "Item" });
diffShapes(shape, inferAll([{ a: "1" }]));
```

## Development

```sh
npm test          # node:test
npm run typecheck
```
