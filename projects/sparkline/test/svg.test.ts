import { test } from "node:test";
import assert from "node:assert/strict";
import { renderSparkline, defaultSpark, esc } from "../src/svg.ts";

test("renders a polyline with one vertex per value", () => {
  const svg = renderSparkline([1, 5, 3, 8], "cpu");
  const pts = /<polyline points="([^"]+)"/.exec(svg)![1].split(" ");
  assert.equal(pts.length, 4);
  assert.match(svg, /^<svg [^>]*aria-label="cpu"/);
});

test("higher values are drawn higher (smaller y)", () => {
  const svg = renderSparkline([0, 10], "x", { ...defaultSpark, markers: false });
  const [a, b] = /<polyline points="([^"]+)"/.exec(svg)![1].split(" ").map((p) => Number(p.split(",")[1]));
  assert.ok(b < a);
});

test("empty series renders a placeholder, single value a dot", () => {
  assert.match(renderSparkline([], "x"), /stroke-dasharray/);
  assert.match(renderSparkline([4], "x"), /<circle/);
});

test("area and bars styles", () => {
  assert.match(renderSparkline([1, 2, 3], "x", { ...defaultSpark, style: "area" }), /<polygon/);
  const bars = renderSparkline([1, 2, 3], "x", { ...defaultSpark, style: "bars" });
  assert.equal(bars.match(/<rect /g)!.length, 3);
});

test("flat series stays inside the canvas and markers can be disabled", () => {
  const svg = renderSparkline([5, 5, 5], "x");
  assert.doesNotMatch(svg, /fill="#dc2626"/);
  assert.match(svg, /,16 /);
});

test("escapes titles and colors", () => {
  assert.equal(esc(`<a href="x">&</a>`), "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;");
  assert.doesNotMatch(renderSparkline([1], `"><script>`, defaultSpark), /<script>/);
});
