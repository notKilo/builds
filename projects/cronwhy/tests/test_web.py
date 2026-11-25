import json
import threading
import urllib.request
from datetime import datetime
from http.server import ThreadingHTTPServer
from zoneinfo import ZoneInfo

from cronwhy.web import Handler, api_explain

NOW = datetime(2026, 1, 1, 10, 7, tzinfo=ZoneInfo("UTC"))


def test_api_ok():
    status, body = api_explain("0 12 * * *", "UTC", 2, NOW)
    assert status == 200
    assert body["explanation"] == "At 12:00"
    assert body["runs"] == ["Thu 2026-01-01 12:00 UTC", "Fri 2026-01-02 12:00 UTC"]


def test_api_errors():
    assert api_explain("nope", "UTC", 1)[0] == 400
    assert "timezone" in api_explain("* * * * *", "Mars/Base", 1)[1]["error"]


def test_server_roundtrip():
    srv = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{srv.server_address[1]}"
    try:
        assert b"cronwhy" in urllib.request.urlopen(base + "/").read()
        data = json.load(urllib.request.urlopen(base + "/api/explain?expr=*/5+*+*+*+*&n=2"))
        assert data["explanation"] == "Every 5 minutes" and len(data["runs"]) == 2
    finally:
        srv.shutdown()
