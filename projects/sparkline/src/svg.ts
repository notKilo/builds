export type Style = "line" | "area" | "bars";

export interface SparkOptions {
  width: number;
  height: number;
  style: Style;
  color: string;
  /** Mark the min and max points with dots. */
  markers: boolean;
  background?: string;
}

export const defaultSpark: SparkOptions = {
  width: 160,
  height: 32,
  style: "line",
  color: "#2563eb",
  markers: true,
};

const PAD = 3;

function fmt(n: number): string {
  return String(Math.round(n * 100) / 100);
}

/** Escape text for use in XML attributes and content. */
export function esc(s: string): string {
  return s.replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!);
}

/** Render values (oldest first) as a standalone SVG sparkline. */
export function renderSparkline(values: number[], title: string, o: SparkOptions = defaultSpark): string {
  const { width: w, height: h } = o;
  const head = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" role="img" aria-label="${esc(title)}"><title>${esc(title)}</title>`;
  const bg = o.background ? `<rect width="${w}" height="${h}" fill="${esc(o.background)}"/>` : "";
  if (values.length === 0) {
    return `${head}${bg}<line x1="${PAD}" y1="${h / 2}" x2="${w - PAD}" y2="${h / 2}" stroke="#9ca3af" stroke-dasharray="2 2"/></svg>`;
  }
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const x = (i: number) => (values.length === 1 ? w / 2 : PAD + (i * (w - 2 * PAD)) / (values.length - 1));
  // A flat series is drawn mid-height rather than on the floor.
  const y = (v: number) => (max === min ? h / 2 : h - PAD - ((v - min) / span) * (h - 2 * PAD));
  const c = esc(o.color);
  let body = "";
  if (o.style === "bars") {
    const bw = Math.max(1, (w - 2 * PAD) / values.length - 1);
    body = values
      .map((v, i) => {
        const top = y(v);
        const bx = PAD + (i * (w - 2 * PAD)) / values.length;
        return `<rect x="${fmt(bx)}" y="${fmt(top)}" width="${fmt(bw)}" height="${fmt(Math.max(1, h - PAD - top))}" fill="${c}"/>`;
      })
      .join("");
  } else {
    const pts = values.map((v, i) => `${fmt(x(i))},${fmt(y(v))}`);
    if (o.style === "area") {
      const area = `${fmt(x(0))},${h - PAD} ${pts.join(" ")} ${fmt(x(values.length - 1))},${h - PAD}`;
      body += `<polygon points="${area}" fill="${c}" fill-opacity="0.2"/>`;
    }
    body += values.length === 1
      ? `<circle cx="${fmt(x(0))}" cy="${fmt(y(values[0]))}" r="1.5" fill="${c}"/>`
      : `<polyline points="${pts.join(" ")}" fill="none" stroke="${c}" stroke-width="1.5" stroke-linejoin="round" stroke-linecap="round"/>`;
  }
  if (o.markers && o.style !== "bars" && max !== min) {
    const iMin = values.indexOf(min);
    const iMax = values.indexOf(max);
    body += `<circle cx="${fmt(x(iMin))}" cy="${fmt(y(min))}" r="2" fill="#dc2626"/>`;
    body += `<circle cx="${fmt(x(iMax))}" cy="${fmt(y(max))}" r="2" fill="#16a34a"/>`;
  }
  return `${head}${bg}${body}</svg>`;
}
