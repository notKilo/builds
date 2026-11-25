# slidedown

Turn one Markdown file into a slide deck. Zero runtime dependencies: the Markdown renderer is written from scratch, the dev server uses `node:http`, and PDF export uses Playwright only if you install it.

## Install

Needs Node 22.18+ (runs TypeScript directly).

```sh
cd projects/slidedown
npm install
```

## Usage

```sh
node src/cli.ts build talk.md -o talk.html   # self-contained HTML file
node src/cli.ts serve talk.md -p 3000        # dev server, reloads on save
node src/cli.ts pdf talk.md -o talk.pdf      # needs: npm i playwright && npx playwright install chromium
```

Example `talk.md`:

```markdown
---
title: My talk
theme: dark
---
# Hello

A first slide with **bold** text.

Notes:
Say hi and introduce yourself.

---
<!-- layout: center -->
## Thanks

- [x] Slides
- [ ] Questions
```

- Slides are split on `---` lines (not inside code fences).
- Front matter keys: `title`, `theme` (`light`, `dark`, `sepia`).
- Per-slide directives: `<!-- layout: default|center|split|title -->`, `<!-- class: extra-class -->`.
- Everything after a `Notes:` line is speaker notes.
- In the browser: arrow keys / space / swipe to navigate, `n` toggles notes, `Home`/`End` jump.

Supported Markdown: headings, paragraphs, emphasis, inline code, fenced code, links, images, blockquotes, nested/task lists, tables. Unsafe URL schemes are neutralised and HTML is escaped.

## Development

```sh
npm test
npm run typecheck
```
