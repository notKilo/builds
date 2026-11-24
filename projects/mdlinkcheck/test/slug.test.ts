import { test } from "node:test";
import assert from "node:assert/strict";
import { slugify, anchorSet } from "../src/slug.ts";

test("slugify matches GitHub style", () => {
  assert.equal(slugify("Hello, World!"), "hello-world");
  assert.equal(slugify("`code` and **bold** [link](x)"), "code-and-bold-link");
  assert.equal(slugify("Über uns"), "über-uns");
});

test("duplicate headings get numeric suffixes", () => {
  assert.deepEqual([...anchorSet(["Usage", "Usage", "Usage"])], ["usage", "usage-1", "usage-2"]);
});
