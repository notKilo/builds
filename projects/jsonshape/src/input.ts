/** Parse text as a JSON document or, failing that, as NDJSON (one value per line). */
export function parseSamples(text: string, source = "input"): unknown[] {
  const trimmed = text.trim();
  if (!trimmed) return [];
  try {
    return [JSON.parse(trimmed)];
  } catch (docErr) {
    const lines = trimmed.split(/\r?\n/).filter((l) => l.trim());
    if (lines.length < 2) throw new Error(`${source}: ${(docErr as Error).message}`);
    return lines.map((line, i) => {
      try {
        return JSON.parse(line);
      } catch (e) {
        throw new Error(`${source}:${i + 1}: ${(e as Error).message}`);
      }
    });
  }
}
