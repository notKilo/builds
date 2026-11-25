import { test } from "node:test";
import assert from "node:assert/strict";
import { renderInline } from "../src/inline.ts";

test("emphasis and strong", () => {
  assert.equal(renderInline("a *b* **c** _d_ __e__"), "a <em>b</em> <strong>c</strong> <em>d</em> <strong>e</strong>");
});

test("snake_case words are not emphasised", () => {
  assert.equal(renderInline("my_var_name"), "my_var_name");
});

test("code spans are literal", () => {
  assert.equal(renderInline("`a *b* <i>`"), "<code>a *b* &lt;i&gt;</code>");
});

test("html is escaped", () => {
  assert.equal(renderInline("<script>x</script>"), "&lt;script&gt;x&lt;/script&gt;");
});

test("links and images", () => {
  assert.equal(renderInline("[**hi**](https://a.b/c \"T\")"), '<a href="https://a.b/c" title="T"><strong>hi</strong></a>');
  assert.equal(renderInline("![alt](img.png)"), '<img src="img.png" alt="alt">');
});

test("javascript: urls are neutralised", () => {
  assert.equal(renderInline("[x](javascript:alert(1))"), '<a href="#">x</a>');
});

test("balanced parentheses stay in the url", () => {
  assert.equal(renderInline("[w](https://x.y/a_(b))"), '<a href="https://x.y/a_(b)">w</a>');
});

test("backslash escapes", () => {
  assert.equal(renderInline("\\*not em\\*"), "*not em*");
});

test("strikethrough", () => {
  assert.equal(renderInline("~~gone~~"), "<del>gone</del>");
});
