import path from "node:path";
import type { Problem } from "./types.ts";

export type Format = "text" | "json" | "sarif";

export interface Summary {
  filesChecked: number;
  linksChecked: number;
  problems: Problem[];
}

const RULE_HELP: Record<Problem["rule"], string> = {
  "missing-file": "A relative link points to a file that does not exist.",
  "missing-anchor": "A #fragment does not match any heading in the target file.",
  "broken-url": "An external URL answered with an error status.",
  "unreachable-url": "An external URL could not be reached (DNS, TLS, timeout).",
};

export function formatText(s: Summary, root: string): string {
  const lines = s.problems.map(
    (p) => `${path.relative(root, p.file)}:${p.line}:${p.column} ${p.severity} [${p.rule}] ${p.message}`,
  );
  const errors = s.problems.filter((p) => p.severity === "error").length;
  const warnings = s.problems.length - errors;
  lines.push(
    `${s.linksChecked} links in ${s.filesChecked} files checked: ${errors} error(s), ${warnings} warning(s)`,
  );
  return lines.join("\n") + "\n";
}

export function formatJson(s: Summary, root: string): string {
  const problems = s.problems.map((p) => ({ ...p, file: path.relative(root, p.file) }));
  return JSON.stringify({ filesChecked: s.filesChecked, linksChecked: s.linksChecked, problems }, null, 2) + "\n";
}

/** SARIF 2.1.0, as consumed by GitHub code scanning. */
export function formatSarif(s: Summary, root: string): string {
  const rules = Object.entries(RULE_HELP).map(([id, text]) => ({ id, shortDescription: { text } }));
  const results = s.problems.map((p) => ({
    ruleId: p.rule,
    level: p.severity,
    message: { text: p.message },
    locations: [
      {
        physicalLocation: {
          artifactLocation: { uri: path.relative(root, p.file).split(path.sep).join("/") },
          region: { startLine: p.line, startColumn: p.column },
        },
      },
    ],
  }));
  const log = {
    $schema: "https://json.schemastore.org/sarif-2.1.0.json",
    version: "2.1.0",
    runs: [{ tool: { driver: { name: "mdlinkcheck", version: "0.1.0", rules } }, results }],
  };
  return JSON.stringify(log, null, 2) + "\n";
}

export function format(fmt: Format, s: Summary, root: string): string {
  if (fmt === "json") return formatJson(s, root);
  if (fmt === "sarif") return formatSarif(s, root);
  return formatText(s, root);
}
