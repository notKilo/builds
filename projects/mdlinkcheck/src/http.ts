export interface UrlResult {
  url: string;
  ok: boolean;
  status?: number;
  error?: string;
}

export interface HttpOptions {
  timeoutMs: number;
  concurrency: number;
  userAgent: string;
  fetchImpl?: typeof fetch;
}

export const defaultHttpOptions: HttpOptions = {
  timeoutMs: 10_000,
  concurrency: 8,
  userAgent: "mdlinkcheck/0.1 (+link checker)",
};

/** HEAD first; some servers reject HEAD (403/405/501), so retry those with GET. */
export async function checkUrl(url: string, opts: HttpOptions): Promise<UrlResult> {
  const doFetch = opts.fetchImpl ?? fetch;
  const attempt = async (method: "HEAD" | "GET") => {
    const res = await doFetch(url, {
      method,
      redirect: "follow",
      headers: { "user-agent": opts.userAgent },
      signal: AbortSignal.timeout(opts.timeoutMs),
    });
    await res.body?.cancel();
    return res.status;
  };
  try {
    let status = await attempt("HEAD");
    if ([403, 405, 501].includes(status)) status = await attempt("GET");
    return { url, ok: status >= 200 && status < 400, status };
  } catch (err) {
    const e = err as Error;
    const message = e.name === "TimeoutError" ? "timed out" : (e.cause as Error | undefined)?.message ?? e.message;
    return { url, ok: false, error: message };
  }
}

/** Check many URLs (deduplicated) with bounded concurrency. */
export async function checkUrls(urls: string[], opts: HttpOptions): Promise<Map<string, UrlResult>> {
  const unique = [...new Set(urls)];
  const results = new Map<string, UrlResult>();
  let next = 0;
  const worker = async () => {
    while (next < unique.length) {
      const url = unique[next++];
      results.set(url, await checkUrl(url, opts));
    }
  };
  await Promise.all(Array.from({ length: Math.min(opts.concurrency, unique.length) }, worker));
  return results;
}
