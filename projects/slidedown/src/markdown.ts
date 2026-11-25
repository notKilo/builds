import { escapeHtml, renderInline } from "./inline.ts";

const slug = (s: string): string =>
  s.toLowerCase().replace(/<[^>]+>/g, "").replace(/[^\w\s-]/g, "").trim().replace(/\s+/g, "-");

const LIST_ITEM = /^(\s*)([-*+]|\d+[.)])\s+(.*)$/;

function splitRow(line: string): string[] {
  return line.trim().replace(/^\||\|$/g, "").split("|").map((c) => c.trim());
}

function renderTable(lines: string[]): string {
  const head = splitRow(lines[0]!);
  const aligns = splitRow(lines[1]!).map((c) =>
    c.startsWith(":") && c.endsWith(":") ? "center" : c.endsWith(":") ? "right" : c.startsWith(":") ? "left" : "",
  );
  const cell = (tag: string, text: string, i: number): string =>
    `<${tag}${aligns[i] ? ` style="text-align:${aligns[i]}"` : ""}>${renderInline(text)}</${tag}>`;
  const rows = lines.slice(2).map((l) => `<tr>${splitRow(l).map((c, i) => cell("td", c, i)).join("")}</tr>`);
  return `<table><thead><tr>${head.map((c, i) => cell("th", c, i)).join("")}</tr></thead><tbody>${rows.join("")}</tbody></table>`;
}

/** Render nested lists from consecutive list-item lines (indentation decides nesting). */
function renderList(lines: string[]): string {
  let i = 0;
  const build = (indent: number): string => {
    const ordered = /\d/.test(LIST_ITEM.exec(lines[i]!)![2]!);
    const tag = ordered ? "ol" : "ul";
    let out = `<${tag}>`;
    while (i < lines.length) {
      const m = LIST_ITEM.exec(lines[i]!);
      if (!m) {
        // continuation line of the previous item
        out = out.replace(/<\/li>$/, ` ${renderInline(lines[i]!.trim())}</li>`);
        i++;
        continue;
      }
      const ind = m[1]!.replace(/\t/g, "    ").length;
      if (ind < indent) break;
      if (ind > indent) {
        out = out.replace(/<\/li>$/, `${build(ind)}</li>`);
        continue;
      }
      let text = m[3]!;
      let box = "";
      const task = /^\[([ xX])\]\s+(.*)$/.exec(text);
      if (task) {
        box = `<input type="checkbox" disabled${task[1] !== " " ? " checked" : ""}> `;
        text = task[2]!;
      }
      out += `<li>${box}${renderInline(text)}</li>`;
      i++;
    }
    return out + `</${tag}>`;
  };
  const first = LIST_ITEM.exec(lines[0]!)![1]!.replace(/\t/g, "    ").length;
  return build(first);
}

/** Render block-level Markdown to HTML. */
export function renderMarkdown(source: string): string {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  const out: string[] = [];
  let i = 0;

  const startsBlock = (l: string): boolean =>
    /^(#{1,6}\s|>|`{3,}|~{3,}|\s*([-*+]|\d+[.)])\s)/.test(l) || /^\s*(\*\s*){3,}$/.test(l);

  while (i < lines.length) {
    const line = lines[i]!;
    if (line.trim() === "") {
      i++;
      continue;
    }

    const fence = /^(`{3,}|~{3,})\s*([\w+-]*)/.exec(line);
    if (fence) {
      const body: string[] = [];
      i++;
      while (i < lines.length && !lines[i]!.startsWith(fence[1]!)) body.push(lines[i++]!);
      i++;
      const cls = fence[2] ? ` class="language-${escapeHtml(fence[2])}"` : "";
      out.push(`<pre><code${cls}>${escapeHtml(body.join("\n"))}</code></pre>`);
      continue;
    }

    const h = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(line);
    if (h) {
      const level = h[1]!.length;
      out.push(`<h${level} id="${slug(h[2]!)}">${renderInline(h[2]!)}</h${level}>`);
      i++;
      continue;
    }

    if (line.startsWith(">")) {
      const quote: string[] = [];
      while (i < lines.length && lines[i]!.startsWith(">")) quote.push(lines[i++]!.replace(/^>\s?/, ""));
      out.push(`<blockquote>${renderMarkdown(quote.join("\n"))}</blockquote>`);
      continue;
    }

    if (line.includes("|") && /^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$/.test(lines[i + 1] ?? "")) {
      const rows = [lines[i++]!, lines[i++]!];
      while (i < lines.length && lines[i]!.includes("|") && lines[i]!.trim() !== "") rows.push(lines[i++]!);
      out.push(renderTable(rows));
      continue;
    }

    if (LIST_ITEM.test(line)) {
      const items: string[] = [];
      while (i < lines.length && lines[i]!.trim() !== "" && (LIST_ITEM.test(lines[i]!) || /^\s+\S/.test(lines[i]!))) {
        items.push(lines[i++]!);
      }
      out.push(renderList(items));
      continue;
    }

    const para: string[] = [];
    while (i < lines.length && lines[i]!.trim() !== "" && (para.length === 0 || !startsBlock(lines[i]!))) {
      para.push(lines[i++]!);
    }
    out.push(`<p>${renderInline(para.join("\n"))}</p>`);
  }
  return out.join("\n");
}
