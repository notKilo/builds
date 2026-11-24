import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { extract } from "./extract.ts";
import { AnchorIndex, checkLocalLink, isHttp } from "./local.ts";
import { checkUrlsCached, UrlCache } from "./cache.ts";
import type { HttpOptions } from "./http.ts";
import type { Summary } from "./report.ts";
import type { Problem } from "./types.ts";

export interface RunOptions {
  paths: string[];
  root: string;
  external: boolean;
  http: HttpOptions;
  cache: UrlCache;
}

const SKIP_DIRS = new Set(["node_modules", ".git"]);

/** Expand files and directories into a sorted list of Markdown files. */
export async function collectFiles(paths: string[]): Promise<string[]> {
  const out = new Set<string>();
  const walk = async (p: string): Promise<void> => {
    let entries;
    try {
      entries = await readdir(p, { withFileTypes: true });
    } catch (err) {
      if ((err as NodeJS.ErrnoException).code === "ENOTDIR") {
        out.add(path.resolve(p));
        return;
      }
      throw err;
    }
    for (const e of entries) {
      if (SKIP_DIRS.has(e.name)) continue;
      const full = path.join(p, e.name);
      if (e.isDirectory()) await walk(full);
      else if (/\.(md|markdown)$/i.test(e.name)) out.add(path.resolve(full));
    }
  };
  for (const p of paths) await walk(p);
  return [...out].sort();
}

export async function run(opts: RunOptions): Promise<Summary> {
  const files = await collectFiles(opts.paths);
  const index = new AnchorIndex();
  const problems: Problem[] = [];
  const urlLinks: { file: string; target: string; line: number; column: number }[] = [];
  let linksChecked = 0;

  for (const file of files) {
    const { links } = extract(await readFile(file, "utf8"));
    for (const link of links) {
      linksChecked++;
      if (isHttp(link.target)) {
        urlLinks.push({ file, target: link.target, line: link.line, column: link.column });
        continue;
      }
      const p = await checkLocalLink(file, link, opts.root, index);
      if (p) problems.push(p);
    }
  }

  if (opts.external && urlLinks.length > 0) {
    await opts.cache.load();
    const results = await checkUrlsCached(urlLinks.map((l) => l.target), opts.http, opts.cache);
    await opts.cache.save();
    for (const l of urlLinks) {
      const r = results.get(l.target)!;
      if (r.ok) continue;
      const broken = r.status !== undefined;
      problems.push({
        file: l.file,
        line: l.line,
        column: l.column,
        target: l.target,
        rule: broken ? "broken-url" : "unreachable-url",
        message: broken ? `HTTP ${r.status} for ${l.target}` : `Could not reach ${l.target}: ${r.error}`,
        severity: broken ? "error" : "warning",
      });
    }
  }

  problems.sort((a, b) => a.file.localeCompare(b.file) || a.line - b.line || a.column - b.column);
  return { filesChecked: files.length, linksChecked, problems };
}
