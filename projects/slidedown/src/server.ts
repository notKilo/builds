import { createServer, type Server, type ServerResponse } from "node:http";
import { readFileSync, watch } from "node:fs";
import { parseDeck } from "./deck.ts";
import { renderDeckHtml } from "./template.ts";

export interface DevServer {
  server: Server;
  port: number;
  close(): void;
}

/** Serve a deck at `/` and push a reload event over SSE whenever the file changes. */
export function startServer(file: string, port: number): Promise<DevServer> {
  const clients = new Set<ServerResponse>();

  const server = createServer((req, res) => {
    if (req.url === "/__reload") {
      res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache", connection: "keep-alive" });
      res.write(": connected\n\n");
      clients.add(res);
      req.on("close", () => clients.delete(res));
      return;
    }
    if (req.url === "/" || req.url?.startsWith("/#") || req.url?.startsWith("/?")) {
      try {
        const html = renderDeckHtml(parseDeck(readFileSync(file, "utf8")), { liveReload: true });
        res.writeHead(200, { "content-type": "text/html; charset=utf-8", "cache-control": "no-store" });
        res.end(html);
      } catch (err) {
        res.writeHead(500, { "content-type": "text/plain" });
        res.end(String(err));
      }
      return;
    }
    res.writeHead(404).end("not found");
  });

  // Editors often fire several events per save; coalesce them.
  let timer: NodeJS.Timeout | undefined;
  const watcher = watch(file, () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      for (const c of clients) c.write("data: reload\n\n");
    }, 50);
  });

  return new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", () => {
      const addr = server.address();
      resolve({
        server,
        port: typeof addr === "object" && addr ? addr.port : port,
        close() {
          watcher.close();
          clearTimeout(timer);
          for (const c of clients) c.end();
          server.close();
        },
      });
    });
  });
}
