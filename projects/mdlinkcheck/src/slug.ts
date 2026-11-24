/** Strip inline Markdown formatting from heading text. */
export function plainText(heading: string): string {
  return heading
    .replace(/!?\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[*_`~]/g, "")
    .replace(/<[^>]+>/g, "")
    .trim();
}

/** GitHub-style slug: lowercase, drop punctuation, spaces to hyphens. */
export function slugify(heading: string): string {
  return plainText(heading)
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .replace(/\s/g, "-");
}

/** Build the set of anchors for a document, de-duplicating like GitHub (-1, -2, ...). */
export function anchorSet(headingTexts: string[]): Set<string> {
  const counts = new Map<string, number>();
  const anchors = new Set<string>();
  for (const text of headingTexts) {
    const base = slugify(text);
    const n = counts.get(base) ?? 0;
    counts.set(base, n + 1);
    anchors.add(n === 0 ? base : `${base}-${n}`);
  }
  return anchors;
}
