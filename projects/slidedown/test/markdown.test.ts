import { test } from "node:test";
import assert from "node:assert/strict";
import { renderMarkdown } from "../src/markdown.ts";

test("headings get ids", () => {
  assert.equal(renderMarkdown("## Hello, World!"), '<h2 id="hello-world">Hello, World!</h2>');
});

test("paragraphs join soft-wrapped lines", () => {
  assert.equal(renderMarkdown("a\nb\n\nc"), "<p>a\nb</p>\n<p>c</p>");
});

test("fenced code is escaped and tagged with language", () => {
  assert.equal(
    renderMarkdown("```ts\nconst a = <b>;\n```"),
    '<pre><code class="language-ts">const a = &lt;b&gt;;</code></pre>',
  );
});

test("unordered and ordered lists", () => {
  assert.equal(renderMarkdown("- a\n- b"), "<ul><li>a</li><li>b</li></ul>");
  assert.equal(renderMarkdown("1. a\n2. b"), "<ol><li>a</li><li>b</li></ol>");
});

test("nested lists", () => {
  assert.equal(
    renderMarkdown("- a\n  - b\n  - c\n- d"),
    "<ul><li>a<ul><li>b</li><li>c</li></ul></li><li>d</li></ul>",
  );
});

test("task list items", () => {
  assert.equal(
    renderMarkdown("- [x] done\n- [ ] todo"),
    '<ul><li><input type="checkbox" disabled checked> done</li><li><input type="checkbox" disabled> todo</li></ul>',
  );
});

test("blockquote renders nested markdown", () => {
  assert.equal(renderMarkdown("> **hi**"), "<blockquote><p><strong>hi</strong></p></blockquote>");
});

test("tables with alignment", () => {
  const html = renderMarkdown("| a | b |\n|:--|--:|\n| 1 | 2 |");
  assert.match(html, /<th style="text-align:left">a<\/th><th style="text-align:right">b<\/th>/);
  assert.match(html, /<td style="text-align:right">2<\/td>/);
});

test("list ends a paragraph", () => {
  assert.equal(renderMarkdown("intro\n- a"), "<p>intro</p>\n<ul><li>a</li></ul>");
});
