export function escapeHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

const SAFE_URL = /^(https?:|mailto:|#|\/|\.{0,2}\/|[\w-]+(\/|\.|$))/i;

function safeUrl(url: string): string {
  const u = url.trim();
  return /^\s*(javascript|data|vbscript):/i.test(u) || !SAFE_URL.test(u) ? "#" : u;
}

/** Render inline Markdown (code, images, links, emphasis) to HTML. Input is untrusted. */
export function renderInline(text: string): string {
  const stash: string[] = [];
  const hold = (html: string): string => `\u0000${stash.push(html) - 1}\u0000`;

  // Code spans first so their contents are never interpreted.
  let s = text.replace(/(`+)([\s\S]*?[^`])\1(?!`)/g, (_, _t: string, code: string) =>
    hold(`<code>${escapeHtml(code.trim())}</code>`),
  );

  // Backslash escapes of punctuation.
  s = s.replace(/\\([\\`*_{}[\]()#+\-.!~|<>])/g, (_, c: string) => hold(escapeHtml(c)));

  // Images then links. Link text is rendered recursively.
  s = s.replace(/!\[([^\]]*)\]\(((?:[^()\s]|\([^()\s]*\))+)(?:\s+"([^"]*)")?\)/g, (_, alt: string, url: string, title?: string) =>
    hold(`<img src="${escapeHtml(safeUrl(url))}" alt="${escapeHtml(alt)}"${title ? ` title="${escapeHtml(title)}"` : ""}>`),
  );
  s = s.replace(/\[([^\]]+)\]\(((?:[^()\s]|\([^()\s]*\))+)(?:\s+"([^"]*)")?\)/g, (_, label: string, url: string, title?: string) =>
    hold(
      `<a href="${escapeHtml(safeUrl(url))}"${title ? ` title="${escapeHtml(title)}"` : ""}>${renderInline(label)}</a>`,
    ),
  );

  s = escapeHtml(s);
  s = s.replace(/\*\*(?=\S)([\s\S]*?\S)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/(^|[^\w])__(?=\S)([\s\S]*?\S)__(?!\w)/g, "$1<strong>$2</strong>");
  s = s.replace(/\*(?=\S)([^*]*?\S)\*/g, "<em>$1</em>");
  s = s.replace(/(^|[^\w])_(?=\S)([^_]*?\S)_(?!\w)/g, "$1<em>$2</em>");
  s = s.replace(/~~(?=\S)([\s\S]*?\S)~~/g, "<del>$1</del>");
  s = s.replace(/ {2,}\n/g, "<br>\n");

  return s.replace(/\u0000(\d+)\u0000/g, (_, i: string) => stash[Number(i)]!);
}
