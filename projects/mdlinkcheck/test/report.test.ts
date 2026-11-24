import { test } from "node:test";
import assert from "node:assert/strict";
import { formatText, formatJson, formatSarif } from "../src/report.ts";
import type { Summary } from "../src/report.ts";

const summary: Summary = {
  filesChecked: 2,
  linksChecked: 5,
  problems: [
    { file: "/repo/docs/a.md", line: 3, column: 7, target: "b.md", rule: "missing-file", message: "File not found: b.md", severity: "error" },
    { file: "/repo/README.md", line: 1, column: 1, target: "http://x", rule: "unreachable-url", message: "timed out", severity: "warning" },
  ],
};

test("text output lists problems and a summary line", () => {
  const out = formatText(summary, "/repo");
  assert.match(out, /^docs\/a\.md:3:7 error \[missing-file\] File not found: b\.md$/m);
  assert.match(out, /5 links in 2 files checked: 1 error\(s\), 1 warning\(s\)/);
});

test("json output uses root-relative paths", () => {
  const parsed = JSON.parse(formatJson(summary, "/repo"));
  assert.equal(parsed.problems[0].file, "docs/a.md");
  assert.equal(parsed.linksChecked, 5);
});

test("sarif output has the expected shape", () => {
  const log = JSON.parse(formatSarif(summary, "/repo"));
  assert.equal(log.version, "2.1.0");
  const [r0, r1] = log.runs[0].results;
  assert.equal(r0.ruleId, "missing-file");
  assert.equal(r0.level, "error");
  assert.equal(r0.locations[0].physicalLocation.artifactLocation.uri, "docs/a.md");
  assert.equal(r0.locations[0].physicalLocation.region.startLine, 3);
  assert.equal(r1.level, "warning");
  assert.ok(log.runs[0].tool.driver.rules.some((r: { id: string }) => r.id === "broken-url"));
});
