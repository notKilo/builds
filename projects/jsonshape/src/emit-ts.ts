import { enumValues, type Shape } from "./shape.ts";

const IDENT = /^[A-Za-z_$][A-Za-z0-9_$]*$/;

export interface TsOptions {
  /** Name of the exported root type. */
  name?: string;
  /** Emit `interface` for objects at the root instead of `type`. */
  interfaces?: boolean;
}

function key(k: string): string {
  return IDENT.test(k) ? k : JSON.stringify(k);
}

function indent(s: string, pad: string): string {
  return s.split("\n").map((l, i) => (i === 0 ? l : pad + l)).join("\n");
}

export function tsType(shape: Shape): string {
  switch (shape.kind) {
    case "null":
    case "boolean":
      return shape.kind;
    case "number":
      return "number";
    case "string": {
      const e = enumValues(shape);
      return e ? e.map((v) => JSON.stringify(v)).join(" | ") : "string";
    }
    case "array": {
      if (!shape.items) return "unknown[]";
      const inner = tsType(shape.items);
      return /[|{]/.test(inner) ? `Array<${inner}>` : `${inner}[]`;
    }
    case "object": {
      const keys = Object.keys(shape.props);
      if (keys.length === 0) return "Record<string, unknown>";
      const lines = keys.map((k) => {
        const p = shape.props[k]!;
        const optional = p.seen < shape.count ? "?" : "";
        return `  ${key(k)}${optional}: ${indent(tsType(p.shape), "  ")};`;
      });
      return `{\n${lines.join("\n")}\n}`;
    }
    case "union":
      return shape.options.map(tsType).join(" | ");
  }
}

export function emitTypeScript(shape: Shape | null, opts: TsOptions = {}): string {
  const name = opts.name ?? "Root";
  if (!shape) return `export type ${name} = unknown;\n`;
  const body = tsType(shape);
  if (opts.interfaces && shape.kind === "object" && Object.keys(shape.props).length > 0) {
    return `export interface ${name} ${body}\n`;
  }
  return `export type ${name} = ${body};\n`;
}
