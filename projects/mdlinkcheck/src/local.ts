import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { extract, type Link } from "./extract.ts";
import { anchorSet } from "./slug.ts";
import type { Problem } from "./types.ts";

const EXTERNAL = /^[a-z][a-z0-9+.-]*:/i;
const MARKDOWN = /\.(md|markdown)$/i;

export function isExternal(target: string): boolean {
  return EXTERNAL.test(target) || target.startsWith("//");
}

export function isHttp(target: string): boolean {
  return /^https?:\/\//i.test(target);
}

/** Loads and caches the anchors of Markdown files. */
export class AnchorIndex {
  private cache = new Map<string, Set<string>>();

  async anchors(file: string): Promise<Set<string>> {
    let set = this.cache.get(file);
    if (!set) {
      const { headings } = extract(await readFile(file, "utf8"));
      set = anchorSet(headings.map((h) => h.text));
      this.cache.set(file, set);
    }
    return set;
  }
}

async function exists(p: string): Promise<"file" | "dir" | null> {
  try {
    return (await stat(p)).isDirectory() ? "dir" : "file";
  } catch {
    return null;
  }
}

function safeDecode(s: string): string {
  try {
    return decodeURIComponent(s);
  } catch {
    return s;
  }
}

/** Check one non-HTTP link found in `file`. Returns a problem or null. */
export async function checkLocalLink(
  file: string,
  link: Link,
  root: string,
  index: AnchorIndex,
): Promise<Problem | null> {
  if (isExternal(link.target)) return null;
  const [rawPath, rawFrag] = splitFragment(link.target);
  const fragment = rawFrag === undefined ? undefined : safeDecode(rawFrag);

  let resolved = file;
  if (rawPath !== "") {
    const p = safeDecode(rawPath);
    resolved = p.startsWith("/") ? path.join(root, p) : path.resolve(path.dirname(file), p);
    const kind = await exists(resolved);
    if (!kind) return problem(file, link, "missing-file", `File not found: ${rawPath}`);
    if (kind === "dir") return null; // directory links are fine; anchors on them are not checkable
  }

  if (fragment && fragment !== "top" && MARKDOWN.test(resolved)) {
    const anchors = await index.anchors(resolved);
    if (!anchors.has(fragment.toLowerCase())) {
      return problem(file, link, "missing-anchor", `No heading for anchor #${fragment} in ${path.relative(root, resolved) || path.basename(resolved)}`);
    }
  }
  return null;
}

function splitFragment(target: string): [string, string | undefined] {
  const q = target.indexOf("?");
  const h = target.indexOf("#");
  const pathEnd = [q, h].filter((i) => i >= 0).reduce((a, b) => Math.min(a, b), target.length);
  return [target.slice(0, pathEnd), h >= 0 ? target.slice(h + 1) : undefined];
}

function problem(file: string, link: Link, rule: Problem["rule"], message: string): Problem {
  return { file, line: link.line, column: link.column, target: link.target, rule, message, severity: "error" };
}
