import http from "node:http";
import { Store } from "./store.ts";
import { dashboardHtml } from "./dashboard.ts";
import { aggregate, isAggregation, parseWindow } from "./query.ts";
import { renderSparkline, defaultSpark, type Style } from "./svg.ts";

export interface ParsedPoint {
  name: string;
  value: number;
  at?: number;
}

const NAME_RE = /^[A-Za-z0-9_.:-]{1,100}$/;

/** Parse "name:value" lines (StatsD-ish) or a JSON body of point objects. */
export function parseBody(body: string, contentType: string): ParsedPoint[] {
  if (contentType.includes("json")) {
    const data = JSON.parse(body);
    const list: unknown[] = Array.isArray(data) ? data : [data];
    return list.map((p) => {
      const o = p as { name?: unknown; value?: unknown; at?: unknown };
      if (typeof o.name !== "string" || typeof o.value !== "number") throw new Error("point needs string name and number value");
      return { name: o.name, value: o.value, at: typeof o.at === "number" ? o.at : undefined };
    });
  }
  return body
    .split("\n")
    .map((l) => l.trim())
    .filter(Boolean)
    .map((line) => {
      const i = line.lastIndexOf(":");
      const value = Number(line.slice(i + 1));
      if (i < 1 || line.slice(i + 1).trim() === "" || Number.isNaN(value)) throw new Error(`bad line: ${line}`);
      return { name: line.slice(0, i), value };
    });
}

async function readBody(req: http.IncomingMessage, limit = 1 << 20): Promise<string> {
  let size = 0;
  const chunks: Buffer[] = [];
  for await (const c of req) {
    size += (c as Buffer).length;
    if (size > limit) throw Object.assign(new Error("body too large"), { status: 413 });
    chunks.push(c as Buffer);
  }
  return Buffer.concat(chunks).toString("utf8");
}

export function createServer(store: Store, now: () => number = Date.now): http.Server {
  return http.createServer(async (req, res) => {
    const url = new URL(req.url ?? "/", "http://localhost");
    const send = (status: number, type: string, body: string, extra: Record<string, string> = {}) => {
      res.writeHead(status, { "content-type": type, ...extra });
      res.end(body);
    };
    const json = (status: number, v: unknown) => send(status, "application/json", JSON.stringify(v));
    try {
      if (req.method === "POST" && url.pathname === "/ingest") {
        const points = parseBody(await readBody(req), req.headers["content-type"] ?? "");
        for (const p of points) {
          if (!NAME_RE.test(p.name)) throw new Error(`invalid series name: ${p.name}`);
        }
        for (const p of points) store.record(p.name, p.value, p.at ?? now());
        return json(202, { accepted: points.length });
      }
      if (req.method === "GET" && url.pathname === "/") {
        return send(200, "text/html; charset=utf-8", dashboardHtml);
      }
      if (req.method === "GET" && url.pathname === "/api/series") {
        return json(200, store.names());
      }
      const api = /^\/api\/series\/([^/]+)$/.exec(url.pathname);
      if (req.method === "GET" && api) {
        const name = decodeURIComponent(api[1]);
        if (!store.has(name)) return json(404, { error: "unknown series" });
        const agg = url.searchParams.get("agg") ?? "avg";
        if (!isAggregation(agg)) throw new Error(`unknown aggregation: ${agg}`);
        const w = url.searchParams.get("window");
        const from = w ? now() - parseWindow(w) : -Infinity;
        const points = store.points(name, from);
        return json(200, { name, agg, value: aggregate(points, agg), buckets: points.length, points });
      }
      const spark = /^\/spark\/([^/]+)\.svg$/.exec(url.pathname);
      if (req.method === "GET" && spark) {
        const name = decodeURIComponent(spark[1]);
        if (!store.has(name)) return json(404, { error: "unknown series" });
        const q = url.searchParams;
        const style = (["line", "area", "bars"].includes(q.get("style") ?? "") ? q.get("style") : "line") as Style;
        const clamp = (v: string | null, d: number, lo: number, hi: number) => {
          const n = Number(v);
          return v && Number.isFinite(n) ? Math.min(hi, Math.max(lo, n)) : d;
        };
        const windowMs = clamp(q.get("window"), Infinity, 1, Infinity);
        const values = store.points(name, now() - windowMs).map((p) => p.sum / p.count);
        const color = /^[0-9a-fA-F]{3,8}$/.test(q.get("color") ?? "") ? `#${q.get("color")}` : defaultSpark.color;
        const svg = renderSparkline(values, name, {
          ...defaultSpark,
          style,
          color,
          width: clamp(q.get("w"), defaultSpark.width, 20, 1000),
          height: clamp(q.get("h"), defaultSpark.height, 10, 400),
          markers: q.get("markers") !== "0",
        });
        return send(200, "image/svg+xml", svg, { "cache-control": "no-cache" });
      }
      json(404, { error: "not found" });
    } catch (e) {
      const err = e as Error & { status?: number };
      json(err.status ?? 400, { error: err.message });
    }
  });
}
