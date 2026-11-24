export type LinkKind = "inline" | "reference" | "autolink" | "image";

export interface Link {
  kind: LinkKind;
  target: string;
  line: number; // 1-based
  column: number; // 1-based
}

export interface Heading {
  level: number;
  text: string;
  line: number;
}

export interface Extracted {
  links: Link[];
  headings: Heading[];
}

const FENCE = /^ {0,3}(`{3,}|~{3,})/;
const HEADING = /^ {0,3}(#{1,6})[ \t]+(.*?)(?:[ \t]+#+)?[ \t]*$/;
const REF_DEF = /^ {0,3}\[([^\]]+)\]:[ \t]*<?([^\s>]+)>?/;
// ![alt](target "title") or [text](target "title"); target may be <wrapped>
const INLINE = /(!?)\[(?:[^\[\]\\]|\\.)*\]\(\s*(<[^>]*>|[^\s)]*)(?:\s+(?:"[^"]*"|'[^']*'))?\s*\)/g;
const REF_USE = /\[(?:[^\[\]\\]|\\.)*\]\[([^\]]*)\]/g;
const AUTOLINK = /<((?:https?|mailto):[^>\s]+)>/g;

/** Blank out inline code spans so links inside them are ignored, keeping columns stable. */
function maskInlineCode(line: string): string {
  return line.replace(/(`+)(.+?)\1/g, (m) => " ".repeat(m.length));
}

export function extract(markdown: string): Extracted {
  const links: Link[] = [];
  const headings: Heading[] = [];
  const refs = new Map<string, { target: string; line: number; column: number }>();
  const used: { label: string; line: number; column: number }[] = [];

  let fence: string | null = null;
  const lines = markdown.split(/\r?\n/);

  lines.forEach((raw, i) => {
    const lineNo = i + 1;
    const f = FENCE.exec(raw);
    if (fence) {
      if (f && f[1][0] === fence[0] && f[1].length >= fence.length) fence = null;
      return;
    }
    if (f) {
      fence = f[1];
      return;
    }

    const h = HEADING.exec(raw);
    if (h) headings.push({ level: h[1].length, text: h[2], line: lineNo });

    const def = REF_DEF.exec(raw);
    if (def) {
      refs.set(def[1].toLowerCase(), {
        target: def[2],
        line: lineNo,
        column: raw.indexOf(def[2]) + 1,
      });
      return;
    }

    const line = maskInlineCode(raw);
    for (const m of line.matchAll(INLINE)) {
      let target = m[2];
      if (target.startsWith("<")) target = target.slice(1, -1);
      if (!target) continue;
      links.push({
        kind: m[1] ? "image" : "inline",
        target,
        line: lineNo,
        column: m.index! + m[0].indexOf(m[2]) + 1,
      });
    }
    for (const m of line.matchAll(AUTOLINK)) {
      links.push({ kind: "autolink", target: m[1], line: lineNo, column: m.index! + 2 });
    }
    for (const m of line.matchAll(REF_USE)) {
      const label = (m[1] || m[0].slice(1, m[0].indexOf("]"))).toLowerCase();
      used.push({ label, line: lineNo, column: m.index! + 1 });
    }
  });

  for (const u of used) {
    const def = refs.get(u.label);
    if (def) links.push({ kind: "reference", target: def.target, line: def.line, column: def.column });
  }
  // Report each reference definition once even if used several times.
  const seen = new Set<string>();
  const unique = links.filter((l) => {
    if (l.kind !== "reference") return true;
    const key = `${l.line}:${l.column}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  unique.sort((a, b) => a.line - b.line || a.column - b.column);
  return { links: unique, headings };
}
