export interface Point {
  t: number; // bucket start, ms epoch
  count: number;
  sum: number;
  min: number;
  max: number;
}

export interface StoreOptions {
  /** Bucket width in milliseconds. */
  resolutionMs: number;
  /** Max buckets kept per series; oldest are dropped. */
  capacity: number;
}

/** In-memory series with fixed-size, time-bucketed ring buffers. */
export class Store {
  readonly opts: StoreOptions;
  private series = new Map<string, Point[]>();

  constructor(opts: Partial<StoreOptions> = {}) {
    this.opts = { resolutionMs: 1000, capacity: 300, ...opts };
  }

  record(name: string, value: number, at: number): void {
    if (!Number.isFinite(value)) throw new RangeError("value must be finite");
    const t = Math.floor(at / this.opts.resolutionMs) * this.opts.resolutionMs;
    const buf = this.series.get(name) ?? [];
    this.series.set(name, buf);
    // Find the bucket from the end; late points are usually near the tail.
    let i = buf.length - 1;
    while (i >= 0 && buf[i].t > t) i--;
    if (i >= 0 && buf[i].t === t) {
      const p = buf[i];
      p.count++;
      p.sum += value;
      p.min = Math.min(p.min, value);
      p.max = Math.max(p.max, value);
      return;
    }
    buf.splice(i + 1, 0, { t, count: 1, sum: value, min: value, max: value });
    if (buf.length > this.opts.capacity) buf.shift();
  }

  names(): string[] {
    return [...this.series.keys()].sort();
  }

  /** Points in [from, to], oldest first. */
  points(name: string, from = -Infinity, to = Infinity): Point[] {
    return (this.series.get(name) ?? []).filter((p) => p.t >= from && p.t <= to).map((p) => ({ ...p }));
  }

  has(name: string): boolean {
    return this.series.has(name);
  }

  /** Serializable copy of all series. */
  snapshot(): Snapshot {
    return { version: 1, opts: this.opts, series: Object.fromEntries(this.series) };
  }

  /** Replace contents with a snapshot; extra points beyond capacity are trimmed from the front. */
  restore(snap: Snapshot): void {
    if (snap.version !== 1) throw new Error(`unsupported snapshot version: ${snap.version}`);
    this.series.clear();
    for (const [name, pts] of Object.entries(snap.series)) {
      this.series.set(name, pts.slice(-this.opts.capacity));
    }
  }
}

export interface Snapshot {
  version: 1;
  opts: StoreOptions;
  series: Record<string, Point[]>;
}
