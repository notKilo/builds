import { parseDeck } from "./deck.ts";
import { renderDeckHtml } from "./template.ts";

/** Resolve the optional `playwright` dependency, with a clear error when it is missing. */
async function loadPlaywright(): Promise<any> {
  try {
    return await import("playwright" as string);
  } catch {
    throw new Error("PDF export needs Playwright: run `npm install playwright` and `npx playwright install chromium`");
  }
}

/** Print every slide to one page of a landscape PDF using headless Chromium. */
export async function exportPdf(source: string, outFile: string): Promise<void> {
  const { chromium } = await loadPlaywright();
  const browser = await chromium.launch();
  try {
    const page = await browser.newPage();
    await page.setContent(renderDeckHtml(parseDeck(source)), { waitUntil: "load" });
    await page.pdf({ path: outFile, width: "1280px", height: "720px", printBackground: true });
  } finally {
    await browser.close();
  }
}
