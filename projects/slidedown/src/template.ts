import type { Deck } from "./deck.ts";
import { escapeHtml } from "./inline.ts";
import { renderMarkdown } from "./markdown.ts";

export const THEMES: Record<string, Record<string, string>> = {
  light: { bg: "#ffffff", fg: "#1c1e21", accent: "#2563eb", code: "#f3f4f6" },
  dark: { bg: "#111827", fg: "#f3f4f6", accent: "#60a5fa", code: "#1f2937" },
  sepia: { bg: "#f6efe0", fg: "#3b2f1e", accent: "#a0522d", code: "#ece2cb" },
};

export interface RenderOptions {
  /** Inject the live-reload client (used by the dev server). */
  liveReload?: boolean;
}

const LAYOUTS = new Set(["default", "center", "split", "title"]);

function themeCss(name: string): string {
  const t = THEMES[name] ?? THEMES.light!;
  return Object.entries(t)
    .map(([k, v]) => `--${k}:${v};`)
    .join("");
}

const CSS = `
*{box-sizing:border-box}
html,body{margin:0;height:100%;background:var(--bg);color:var(--fg);font:28px/1.4 system-ui,sans-serif}
.slide{display:none;position:absolute;inset:0;padding:6vh 8vw;overflow:auto}
.slide.active{display:block}
.slide.center{display:none;flex-direction:column;justify-content:center;text-align:center}
.slide.center.active,.slide.title.active{display:flex}
.slide.title{display:none;flex-direction:column;justify-content:center;font-size:1.3em}
.slide.split.active{display:grid;grid-template-columns:1fr 1fr;gap:4vw;align-content:start}
a{color:var(--accent)}
pre,code{background:var(--code);border-radius:6px;font:0.8em ui-monospace,monospace}
code{padding:.1em .3em}pre{padding:.8em;overflow:auto}pre code{padding:0}
blockquote{margin:1em 0;padding-left:1em;border-left:4px solid var(--accent)}
table{border-collapse:collapse}th,td{border:1px solid var(--code);padding:.3em .7em}
img{max-width:100%}
#counter{position:fixed;right:1rem;bottom:.6rem;font-size:14px;opacity:.6}
#notes{display:none;position:fixed;left:0;right:0;bottom:0;max-height:35vh;overflow:auto;padding:1rem 2rem;background:var(--code);font-size:18px;border-top:2px solid var(--accent)}
#notes.show{display:block}
@media print{.slide{display:block!important;position:relative;page-break-after:always;height:100vh}#counter,#notes{display:none!important}}
`;

const SCRIPT = `
(function(){
  var slides=[].slice.call(document.querySelectorAll('.slide'));
  var notes=document.getElementById('notes'),counter=document.getElementById('counter');
  var i=Math.max(0,Math.min(slides.length-1,(parseInt(location.hash.slice(1),10)||1)-1));
  function show(n){
    i=Math.max(0,Math.min(slides.length-1,n));
    slides.forEach(function(s,k){s.classList.toggle('active',k===i)});
    notes.textContent=slides[i].getAttribute('data-notes')||'';
    counter.textContent=(i+1)+' / '+slides.length;
    history.replaceState(null,'','#'+(i+1));
  }
  document.addEventListener('keydown',function(e){
    var k=e.key;
    if(k==='ArrowRight'||k==='PageDown'||k===' '||k==='Enter')show(i+1);
    else if(k==='ArrowLeft'||k==='PageUp'||k==='Backspace')show(i-1);
    else if(k==='Home')show(0);
    else if(k==='End')show(slides.length-1);
    else if(k==='n')notes.classList.toggle('show');
    else return;
    e.preventDefault();
  });
  var x0=null;
  document.addEventListener('touchstart',function(e){x0=e.touches[0].clientX});
  document.addEventListener('touchend',function(e){
    if(x0===null)return;var dx=e.changedTouches[0].clientX-x0;x0=null;
    if(Math.abs(dx)>50)show(i+(dx<0?1:-1));
  });
  show(i);
})();
`;

const LIVE_RELOAD = `<script>new EventSource('/__reload').onmessage=function(){location.reload()}</script>`;

/** Render a parsed deck to one self-contained HTML page. */
export function renderDeckHtml(deck: Deck, opts: RenderOptions = {}): string {
  const title = deck.meta.title ?? "Slides";
  const theme = deck.meta.theme ?? "light";
  const sections = deck.slides.map((s) => {
    const layout = LAYOUTS.has(s.directives.layout ?? "") ? s.directives.layout : "default";
    const cls = ["slide", layout, ...(s.directives.class ? [s.directives.class.replace(/[^\w -]/g, "")] : [])];
    return `<section class="${cls.join(" ")}" data-notes="${escapeHtml(s.notes)}">\n${renderMarkdown(s.markdown)}\n</section>`;
  });
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>${escapeHtml(title)}</title>
<style>:root{${themeCss(theme)}}${CSS}</style>
</head>
<body>
${sections.join("\n")}
<div id="notes"></div>
<div id="counter"></div>
<script>${SCRIPT}</script>
${opts.liveReload ? LIVE_RELOAD : ""}
</body>
</html>
`;
}
