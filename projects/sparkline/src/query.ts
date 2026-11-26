import type { Point } from "./store.ts";

export const AGGREGATIONS = ["avg", "min", "max", "sum", "count", "p95"] as const;
export type Aggregation = (typeof AGGREGATIONS)[number];

export function isAggregation(s: string): s is Aggregation {
  return (AGGREGATIONS as readonly string[]).includes(s);
}

/**
 * Aggregate bucketed points into one number. Returns null for no data.
 * p95 is approximated from bucket means, since raw samples are not kept.
 */
export function aggregate(points: Point[], agg: Aggregation): number | null {
  if (points.length === 0) return null;
  switch (agg) {
    case "min":
      return Math.min(...points.map((p) => p.min));
    case "max":
      return Math.max(...points.map((p) => p.max));
    case "sum":
      return points.reduce((a, p) => a + p.sum, 0);
    case "count":
      return points.reduce((a, p) => a + p.count, 0);
    case "avg": {
      const count = points.reduce((a, p) => a + p.count, 0);
      return points.reduce((a, p) => a + p.sum, 0) / count;
    }
    case "p95":
      return percentile(points.map((p) => p.sum / p.count), 95);
  }
}

/** Nearest-rank percentile. */
export function percentile(values: number[], pct: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  const rank = Math.ceil((pct / 100) * sorted.length);
  return sorted[Math.max(0, rank - 1)];
}

/** Parse durations like "30s", "5m", "2h", "1d" or plain milliseconds. */
export function parseWindow(s: string): number {
  const m = /^(\d+(?:\.\d+)?)(ms|s|m|h|d)?$/.exec(s.trim());
  if (!m) throw new Error(`bad window: ${s}`);
  const unit = { ms: 1, s: 1000, m: 60_000, h: 3_600_000, d: 86_400_000 }[m[2] ?? "ms"]!;
  const ms = Number(m[1]) * unit;
  if (ms <= 0) throw new Error(`bad window: ${s}`);
  return ms;
}
