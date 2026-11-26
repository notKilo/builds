import { enumValues, type Shape } from "./shape.ts";

/**
 * A change is breaking when code written against the old shape can fail on
 * data of the new shape (a consumer's point of view).
 */
export interface Change {
  /** JSON-path-like location, e.g. `$.user.tags[]`. */
  path: string;
  kind:
    | "prop-added"
    | "prop-removed"
    | "became-optional"
    | "became-required"
    | "type-added"
    | "type-removed"
    | "integer-widened"
    | "integer-narrowed"
    | "enum-changed"
    | "format-changed";
  breaking: boolean;
  message: string;
}

type Kind = Shape["kind"];

/** Flatten a shape to its non-union options. */
function options(s: Shape | null): Shape[] {
  if (!s) return [];
  return s.kind === "union" ? s.options : [s];
}

function label(s: Shape): string {
  return s.kind === "number" && s.integer ? "integer" : s.kind;
}

function fmtValues(v: string[] | undefined): string {
  return v ? v.map((x) => JSON.stringify(x)).join(", ") : "any string";
}

export function diffShapes(oldShape: Shape | null, newShape: Shape | null): Change[] {
  const changes: Change[] = [];
  walk(oldShape, newShape, "$", changes);
  return changes;
}

function walk(a: Shape | null, b: Shape | null, path: string, out: Change[]): void {
  const oldOpts = options(a);
  const newOpts = options(b);
  const oldByKind = new Map<Kind, Shape>(oldOpts.map((o) => [o.kind, o]));
  const newByKind = new Map<Kind, Shape>(newOpts.map((o) => [o.kind, o]));

  for (const [kind, n] of newByKind) {
    if (!oldByKind.has(kind) && oldOpts.length > 0) {
      out.push({ path, kind: "type-added", breaking: true, message: `now may also be ${label(n)}` });
    }
  }
  for (const [kind, o] of oldByKind) {
    if (!newByKind.has(kind) && newOpts.length > 0) {
      out.push({ path, kind: "type-removed", breaking: false, message: `no longer ${label(o)}` });
    }
  }
  for (const [kind, o] of oldByKind) {
    const n = newByKind.get(kind);
    if (n) diffSame(o, n, path, out);
  }
}

function diffSame(a: Shape, b: Shape, path: string, out: Change[]): void {
  if (a.kind === "number" && b.kind === "number") {
    if (a.integer && !b.integer) {
      out.push({ path, kind: "integer-widened", breaking: true, message: "integer is now a non-integer number" });
    } else if (!a.integer && b.integer) {
      out.push({ path, kind: "integer-narrowed", breaking: false, message: "number is now always an integer" });
    }
  } else if (a.kind === "string" && b.kind === "string") {
    diffStrings(a, b, path, out);
  } else if (a.kind === "array" && b.kind === "array") {
    walk(a.items, b.items, `${path}[]`, out);
  } else if (a.kind === "object" && b.kind === "object") {
    diffObjects(a, b, path, out);
  }
}

function diffStrings(
  a: Extract<Shape, { kind: "string" }>,
  b: Extract<Shape, { kind: "string" }>,
  path: string,
  out: Change[],
): void {
  if (a.format !== b.format) {
    out.push({
      path,
      kind: "format-changed",
      breaking: true,
      message: `format ${a.format ?? "none"} -> ${b.format ?? "none"}`,
    });
  }
  const ea = enumValues(a);
  const eb = enumValues(b);
  if (!ea && !eb) return;
  const sameValues = ea && eb && ea.length === eb.length && ea.every((v) => eb.includes(v));
  if (sameValues) return;
  // Widening an enum (or dropping it) can surface values consumers never handled.
  const added = !eb || !ea ? !eb : eb.some((v) => !ea.includes(v));
  out.push({
    path,
    kind: "enum-changed",
    breaking: added,
    message: `values ${fmtValues(ea)} -> ${fmtValues(eb)}`,
  });
}

function diffObjects(
  a: Extract<Shape, { kind: "object" }>,
  b: Extract<Shape, { kind: "object" }>,
  path: string,
  out: Change[],
): void {
  const required = (s: typeof a, k: string) => s.props[k]!.seen === s.count;
  for (const key of Object.keys(a.props)) {
    const p = `${path}.${key}`;
    if (!(key in b.props)) {
      const req = required(a, key);
      out.push({
        path: p,
        kind: "prop-removed",
        breaking: req,
        message: req ? "required property removed" : "optional property removed",
      });
      continue;
    }
    const wasReq = required(a, key);
    const isReq = required(b, key);
    if (wasReq && !isReq) {
      out.push({ path: p, kind: "became-optional", breaking: true, message: "required -> optional" });
    } else if (!wasReq && isReq) {
      out.push({ path: p, kind: "became-required", breaking: false, message: "optional -> required" });
    }
    walk(a.props[key]!.shape, b.props[key]!.shape, p, out);
  }
  for (const key of Object.keys(b.props)) {
    if (key in a.props) continue;
    const req = required(b, key);
    out.push({
      path: `${path}.${key}`,
      kind: "prop-added",
      breaking: false,
      message: req ? "required property added" : "optional property added",
    });
  }
}

export function formatChanges(changes: Change[]): string {
  if (changes.length === 0) return "No differences.\n";
  const sorted = [...changes].sort((x, y) => Number(y.breaking) - Number(x.breaking));
  const lines = sorted.map((c) => `${c.breaking ? "BREAKING    " : "non-breaking"}  ${c.path}  ${c.message}`);
  const breaking = changes.filter((c) => c.breaking).length;
  lines.push("", `${breaking} breaking, ${changes.length - breaking} non-breaking`);
  return lines.join("\n") + "\n";
}
