/** A Shape describes the set of JSON values seen so far. */
export type Shape =
  | { kind: "null" }
  | { kind: "boolean" }
  | { kind: "number"; integer: boolean }
  | { kind: "string"; format?: StringFormat; values?: string[]; count: number }
  | { kind: "array"; items: Shape | null }
  | { kind: "object"; props: Record<string, Prop>; count: number }
  | { kind: "union"; options: Shape[] };

export interface Prop {
  shape: Shape;
  /** Number of sample objects that contained this key. */
  seen: number;
}

export type StringFormat = "date-time" | "date" | "uuid" | "email" | "uri";

/** Distinct string values kept for enum inference. */
export const MAX_ENUM_VALUES = 8;
/** An enum needs at least this many samples to be believable. */
export const MIN_ENUM_SAMPLES = 3;

const FORMATS: Array<[StringFormat, RegExp]> = [
  ["date-time", /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/],
  ["date", /^\d{4}-\d{2}-\d{2}$/],
  ["uuid", /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i],
  ["email", /^[^\s@]+@[^\s@]+\.[^\s@]+$/],
  ["uri", /^https?:\/\/[^\s]+$/],
];

export function detectFormat(s: string): StringFormat | undefined {
  return FORMATS.find(([, re]) => re.test(s))?.[0];
}

export function inferShape(value: unknown): Shape {
  if (value === null) return { kind: "null" };
  switch (typeof value) {
    case "boolean":
      return { kind: "boolean" };
    case "number":
      return { kind: "number", integer: Number.isInteger(value) };
    case "string":
      return { kind: "string", format: detectFormat(value), values: [value], count: 1 };
  }
  if (Array.isArray(value)) {
    return {
      kind: "array",
      items: value.map(inferShape).reduce<Shape | null>((acc, s) => (acc ? mergeShapes(acc, s) : s), null),
    };
  }
  const props: Record<string, Prop> = {};
  for (const [k, v] of Object.entries(value as Record<string, unknown>)) {
    props[k] = { shape: inferShape(v), seen: 1 };
  }
  return { kind: "object", props, count: 1 };
}

function mergeStrings(
  a: Extract<Shape, { kind: "string" }>,
  b: Extract<Shape, { kind: "string" }>,
): Shape {
  let values: string[] | undefined;
  if (a.values && b.values) {
    const set = new Set([...a.values, ...b.values]);
    values = set.size <= MAX_ENUM_VALUES ? [...set] : undefined;
  }
  return {
    kind: "string",
    format: a.format === b.format ? a.format : undefined,
    values,
    count: a.count + b.count,
  };
}

function mergeObjects(
  a: Extract<Shape, { kind: "object" }>,
  b: Extract<Shape, { kind: "object" }>,
): Shape {
  const props: Record<string, Prop> = {};
  for (const key of new Set([...Object.keys(a.props), ...Object.keys(b.props)])) {
    const pa = a.props[key];
    const pb = b.props[key];
    props[key] =
      pa && pb
        ? { shape: mergeShapes(pa.shape, pb.shape), seen: pa.seen + pb.seen }
        : { ...(pa ?? pb)! };
  }
  return { kind: "object", props, count: a.count + b.count };
}

/** Merge two shapes into one that accepts every value either accepts. */
export function mergeShapes(a: Shape, b: Shape): Shape {
  if (a.kind === "union" || b.kind === "union") {
    const options = [...(a.kind === "union" ? a.options : [a]), ...(b.kind === "union" ? b.options : [b])];
    return normalizeUnion(options);
  }
  if (a.kind !== b.kind) return normalizeUnion([a, b]);
  switch (a.kind) {
    case "number":
      return { kind: "number", integer: a.integer && (b as typeof a).integer };
    case "string":
      return mergeStrings(a, b as typeof a);
    case "array": {
      const bi = (b as typeof a).items;
      return { kind: "array", items: a.items && bi ? mergeShapes(a.items, bi) : (a.items ?? bi) };
    }
    case "object":
      return mergeObjects(a, b as typeof a);
    default:
      return a;
  }
}

/** Collapse options of the same kind together; unwrap single-option unions. */
function normalizeUnion(options: Shape[]): Shape {
  const merged: Shape[] = [];
  for (const opt of options) {
    const i = merged.findIndex((m) => m.kind === opt.kind);
    if (i >= 0) merged[i] = mergeShapes(merged[i]!, opt);
    else merged.push(opt);
  }
  return merged.length === 1 ? merged[0]! : { kind: "union", options: merged };
}

export function inferAll(samples: unknown[]): Shape | null {
  return samples.map(inferShape).reduce<Shape | null>((acc, s) => (acc ? mergeShapes(acc, s) : s), null);
}

/** True when a string shape has been seen often enough with few distinct values. */
export function enumValues(s: Extract<Shape, { kind: "string" }>): string[] | undefined {
  return s.values && s.count >= MIN_ENUM_SAMPLES && s.values.length < s.count ? s.values : undefined;
}
