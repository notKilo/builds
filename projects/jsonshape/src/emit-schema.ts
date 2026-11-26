import { enumValues, type Shape } from "./shape.ts";

export type JsonSchema = Record<string, unknown>;

export function toSchema(shape: Shape): JsonSchema {
  switch (shape.kind) {
    case "null":
    case "boolean":
      return { type: shape.kind };
    case "number":
      return { type: shape.integer ? "integer" : "number" };
    case "string": {
      const out: JsonSchema = { type: "string" };
      if (shape.format) out.format = shape.format;
      const e = enumValues(shape);
      if (e) out.enum = e;
      return out;
    }
    case "array":
      return shape.items ? { type: "array", items: toSchema(shape.items) } : { type: "array" };
    case "object": {
      const properties: Record<string, JsonSchema> = {};
      const required: string[] = [];
      for (const [k, p] of Object.entries(shape.props)) {
        properties[k] = toSchema(p.shape);
        if (p.seen === shape.count) required.push(k);
      }
      const out: JsonSchema = { type: "object", properties };
      if (required.length) out.required = required;
      return out;
    }
    case "union":
      return { anyOf: shape.options.map(toSchema) };
  }
}

export function emitJsonSchema(shape: Shape | null, title?: string): string {
  const body: JsonSchema = shape ? toSchema(shape) : {};
  const doc: JsonSchema = { $schema: "https://json-schema.org/draft/2020-12/schema", ...(title ? { title } : {}), ...body };
  return JSON.stringify(doc, null, 2) + "\n";
}
