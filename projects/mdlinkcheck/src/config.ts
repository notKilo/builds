import { readFile } from "node:fs/promises";
import path from "node:path";

export interface Config {
  /** Globs (relative to the working directory) of Markdown files to skip. */
  ignorePaths?: string[];
  /** URL prefixes that are never checked, e.g. "http://localhost". */
  ignoreUrls?: string[];
  format?: "text" | "json" | "sarif";
  external?: boolean;
  strict?: boolean;
  timeout?: number;
  concurrency?: number;
  cacheTtl?: number;
}

export const CONFIG_FILE = ".mdlinkcheck.json";

const isStringArray = (v: unknown): v is string[] => Array.isArray(v) && v.every((x) => typeof x === "string");

/** Validate parsed JSON; throws with a readable message on bad shapes. */
export function parseConfig(raw: unknown): Config {
  if (typeof raw !== "object" || raw === null || Array.isArray(raw)) throw new Error("config must be a JSON object");
  const o = raw as Record<string, unknown>;
  const known = ["ignorePaths", "ignoreUrls", "format", "external", "strict", "timeout", "concurrency", "cacheTtl"];
  for (const k of Object.keys(o)) if (!known.includes(k)) throw new Error(`unknown config key "${k}"`);
  for (const k of ["ignorePaths", "ignoreUrls"]) {
    if (k in o && !isStringArray(o[k])) throw new Error(`"${k}" must be an array of strings`);
  }
  if ("format" in o && !["text", "json", "sarif"].includes(o.format as string)) throw new Error(`"format" must be text, json or sarif`);
  for (const k of ["external", "strict"]) {
    if (k in o && typeof o[k] !== "boolean") throw new Error(`"${k}" must be a boolean`);
  }
  for (const k of ["timeout", "concurrency", "cacheTtl"]) {
    if (k in o && !(typeof o[k] === "number" && o[k] > 0)) throw new Error(`"${k}" must be a positive number`);
  }
  return o as Config;
}

/** Load config from an explicit path, or from `.mdlinkcheck.json` in `dir` if present. */
export async function loadConfig(dir: string, explicit?: string): Promise<Config> {
  const file = explicit ? path.resolve(explicit) : path.join(dir, CONFIG_FILE);
  let text: string;
  try {
    text = await readFile(file, "utf8");
  } catch (err) {
    if (!explicit && (err as NodeJS.ErrnoException).code === "ENOENT") return {};
    throw new Error(`cannot read config ${file}: ${(err as Error).message}`);
  }
  try {
    return parseConfig(JSON.parse(text));
  } catch (err) {
    throw new Error(`${file}: ${(err as Error).message}`);
  }
}
