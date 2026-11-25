import { test } from "node:test";
import assert from "node:assert/strict";
import { parseDeck } from "../src/deck.ts";
import { renderDeckHtml } from "../src/template.ts";

const SRC = `---
title: Demo <talk>
theme: dark
---
# One

Notes:
say "hi"
---
<!-- layout: center -->
# Two
`;

test("renders one section per slide with title and theme", () => {
  const html = renderDeckHtml(parseDeck(SRC));
  assert.equal(html.match(/<section /g)?.length, 2);
  assert.match(html, /<title>Demo &lt;talk&gt;<\/title>/);
  assert.match(html, /--bg:#111827/);
});

test("layout directive becomes a class and notes are escaped", () => {
  const html = renderDeckHtml(parseDeck(SRC));
  assert.match(html, /class="slide center"/);
  assert.match(html, /data-notes="say &quot;hi&quot;"/);
});

test("unknown layout and theme fall back safely", () => {
  const html = renderDeckHtml(parseDeck("---\ntheme: nope\n---\n<!-- layout: \"><x -->\n# A"));
  assert.match(html, /class="slide default"/);
  assert.match(html, /--bg:#ffffff/);
});

test("live reload client only injected on request", () => {
  const deck = parseDeck("# A");
  assert.doesNotMatch(renderDeckHtml(deck), /__reload/);
  assert.match(renderDeckHtml(deck, { liveReload: true }), /__reload/);
});
