import { test } from "node:test";
import assert from "node:assert/strict";
import { parseDeck, parseFrontMatter, splitSlides } from "../src/deck.ts";

test("front matter is parsed and removed", () => {
  const { meta, body } = parseFrontMatter('---\ntitle: "Hello"\ntheme: dark\n---\n# A');
  assert.deepEqual(meta, { title: "Hello", theme: "dark" });
  assert.equal(body.trim(), "# A");
});

test("no front matter leaves body intact", () => {
  assert.deepEqual(parseFrontMatter("# A\n---\n# B").meta, {});
});

test("unterminated front matter is treated as content", () => {
  assert.deepEqual(parseFrontMatter("---\ntitle: x\n# A").meta, {});
});

test("slides split on rules", () => {
  assert.deepEqual(splitSlides("# A\n\n---\n\n# B\n---\n# C"), ["# A", "# B", "# C"]);
});

test("rules inside code fences do not split", () => {
  const body = "# A\n```yaml\na: 1\n---\nb: 2\n```\n---\n# B";
  assert.equal(splitSlides(body).length, 2);
});

test("empty slides are dropped", () => {
  assert.deepEqual(splitSlides("# A\n---\n\n---\n# B"), ["# A", "# B"]);
});

test("directives and notes are extracted", () => {
  const deck = parseDeck("# A\n<!-- layout: center -->\ntext\n\nNotes:\nsay hi\n<!-- ignored -->");
  const s = deck.slides[0]!;
  assert.equal(s.directives.layout, "center");
  assert.equal(s.markdown, "# A\ntext");
  assert.equal(s.notes, "say hi\n<!-- ignored -->");
});

test("notes marker inside a code fence is content", () => {
  const s = parseDeck("```\nNotes:\n```").slides[0]!;
  assert.equal(s.notes, "");
});
