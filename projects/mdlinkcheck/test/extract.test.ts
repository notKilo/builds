import { test } from "node:test";
import assert from "node:assert/strict";
import { extract } from "../src/extract.ts";

test("finds inline links and images with positions", () => {
  const { links } = extract("See [docs](docs/a.md) and ![logo](img/l.png \"Logo\").");
  assert.deepEqual(
    links.map((l) => [l.kind, l.target, l.line]),
    [["inline", "docs/a.md", 1], ["image", "img/l.png", 1]],
  );
  assert.equal(links[0].column, 12);
});

test("ignores fenced code and inline code", () => {
  const md = "```\n[x](nope.md)\n```\nUse `[y](nope2.md)` then [z](yes.md)";
  assert.deepEqual(extract(md).links.map((l) => l.target), ["yes.md"]);
});

test("resolves reference links via definitions", () => {
  const md = "[a][r1] and [r2][] again [a][r1]\n\n[r1]: ./one.md\n[R2]: https://x.test/two";
  assert.deepEqual(extract(md).links.map((l) => l.target), ["./one.md", "https://x.test/two"]);
});

test("autolinks and angle-bracket targets", () => {
  const md = "<https://a.test> and [t](<my file.md>)";
  assert.deepEqual(extract(md).links.map((l) => [l.kind, l.target]), [
    ["autolink", "https://a.test"],
    ["inline", "my file.md"],
  ]);
});

test("collects headings, skipping those in fences", () => {
  const md = "# Title\n```\n# not\n```\n## Sub ##\n";
  assert.deepEqual(extract(md).headings.map((h) => [h.level, h.text]), [[1, "Title"], [2, "Sub"]]);
});
