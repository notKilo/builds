export interface Slide {
  markdown: string;
  notes: string;
  directives: Record<string, string>;
}

export interface Deck {
  meta: Record<string, string>;
  slides: Slide[];
}

const FENCE = /^(`{3,}|~{3,})/;
const DIRECTIVE = /^<!--\s*([\w-]+)\s*:\s*(.*?)\s*-->$/;
const NOTES_MARK = /^Notes?:\s*$/i;

/** Parse leading `---` front matter of simple `key: value` lines. */
export function parseFrontMatter(source: string): { meta: Record<string, string>; body: string } {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  if (lines[0]?.trim() !== "---") return { meta: {}, body: lines.join("\n") };
  const end = lines.findIndex((l, i) => i > 0 && l.trim() === "---");
  if (end === -1) return { meta: {}, body: lines.join("\n") };
  const meta: Record<string, string> = {};
  for (const line of lines.slice(1, end)) {
    const m = /^([\w-]+)\s*:\s*(.*)$/.exec(line);
    if (m) meta[m[1]!] = m[2]!.replace(/^["']|["']$/g, "");
  }
  return { meta, body: lines.slice(end + 1).join("\n") };
}

/** Split a document into slides on `---` lines that are outside code fences. */
export function splitSlides(body: string): string[] {
  const chunks: string[][] = [[]];
  let fence: string | null = null;
  for (const line of body.split("\n")) {
    const f = FENCE.exec(line);
    if (f) {
      if (fence === null) fence = f[1]![0]!;
      else if (line.startsWith(fence.repeat(3))) fence = null;
    }
    if (fence === null && /^-{3,}\s*$/.test(line)) chunks.push([]);
    else chunks[chunks.length - 1]!.push(line);
  }
  return chunks.map((c) => c.join("\n").trim()).filter((c) => c.length > 0);
}

/** Extract `<!-- key: value -->` directives and a trailing `Notes:` section. */
export function parseSlide(chunk: string): Slide {
  const directives: Record<string, string> = {};
  const content: string[] = [];
  const notes: string[] = [];
  let inNotes = false;
  let fence: string | null = null;
  for (const line of chunk.split("\n")) {
    const f = FENCE.exec(line);
    if (f && !inNotes) {
      if (fence === null) fence = f[1]![0]!;
      else if (line.startsWith(fence.repeat(3))) fence = null;
    }
    if (fence === null && !inNotes) {
      const d = DIRECTIVE.exec(line.trim());
      if (d) {
        directives[d[1]!] = d[2]!;
        continue;
      }
      if (NOTES_MARK.test(line.trim())) {
        inNotes = true;
        continue;
      }
    }
    (inNotes ? notes : content).push(line);
  }
  return { markdown: content.join("\n").trim(), notes: notes.join("\n").trim(), directives };
}

export function parseDeck(source: string): Deck {
  const { meta, body } = parseFrontMatter(source);
  return { meta, slides: splitSlides(body).map(parseSlide) };
}
