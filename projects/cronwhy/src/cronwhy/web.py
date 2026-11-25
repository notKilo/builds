"""Tiny stdlib web UI: a static page plus a JSON API."""
from __future__ import annotations

import json
from datetime import datetime
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse
from zoneinfo import ZoneInfo, ZoneInfoNotFoundError

from .explain import explain
from .parser import CronError, parse
from .schedule import next_runs

PAGE = """<!doctype html>
<html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>cronwhy</title>
<style>
body{font:16px system-ui,sans-serif;max-width:40rem;margin:2rem auto;padding:0 1rem;color:#222}
input{font:inherit;padding:.4rem;width:100%;box-sizing:border-box;margin:.25rem 0 1rem}
#expr{font-family:monospace;font-size:1.3rem}
#say{font-size:1.2rem;min-height:1.5rem}.err{color:#b00020}
li{font-family:monospace;margin:.2rem 0}
@media(prefers-color-scheme:dark){body{background:#181818;color:#eee}input{background:#222;color:#eee;border:1px solid #555}}
</style>
<h1>cronwhy</h1>
<label>Cron expression<input id="expr" value="*/15 9-17 * * mon-fri" autofocus></label>
<label>Timezone<input id="tz" value="UTC"></label>
<p id="say"></p><ol id="runs"></ol>
<script>
const $=id=>document.getElementById(id);
async function update(){
  const q=new URLSearchParams({expr:$('expr').value,tz:$('tz').value,n:8});
  const r=await fetch('/api/explain?'+q), d=await r.json();
  $('say').textContent=d.error||d.explanation; $('say').className=d.error?'err':'';
  $('runs').replaceChildren(...(d.runs||[]).map(t=>Object.assign(document.createElement('li'),{textContent:t})));
}
$('expr').oninput=$('tz').oninput=update; update();
</script>
</html>
"""


def api_explain(expr: str, tz: str, n: int, now: datetime | None = None) -> tuple[int, dict]:
    try:
        parsed = parse(expr)
        zone = ZoneInfo(tz)
        start = now or datetime.now(zone)
        runs = next_runs(parsed, start, max(1, min(n, 50)), tz)
    except CronError as e:
        return 400, {"error": str(e)}
    except (ZoneInfoNotFoundError, ValueError):
        return 400, {"error": f"unknown timezone {tz!r}"}
    return 200, {
        "expression": expr,
        "explanation": explain(parsed),
        "runs": [r.strftime("%a %Y-%m-%d %H:%M %Z") for r in runs],
    }


class Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        url = urlparse(self.path)
        if url.path == "/":
            self._send(200, PAGE.encode(), "text/html; charset=utf-8")
        elif url.path == "/api/explain":
            q = parse_qs(url.query)
            n = q.get("n", ["5"])[0]
            status, body = api_explain(q.get("expr", [""])[0], q.get("tz", ["UTC"])[0],
                                       int(n) if n.isdigit() else 5)
            self._send(status, json.dumps(body).encode(), "application/json")
        else:
            self._send(404, b"not found", "text/plain")

    def _send(self, status: int, body: bytes, ctype: str) -> None:
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args) -> None:  # keep the console quiet
        pass


def serve(host: str = "127.0.0.1", port: int = 8000) -> None:
    server = ThreadingHTTPServer((host, port), Handler)
    print(f"cronwhy web UI on http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
