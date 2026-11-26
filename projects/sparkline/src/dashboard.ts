/** Single-page dashboard; fetches the series list and refreshes the sparklines. */
export const dashboardHtml = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>sparkline</title>
<style>
  :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
  body { margin: 0; padding: 1rem 1.5rem; }
  h1 { font-size: 1.1rem; margin: 0 0 1rem; }
  .bar { display: flex; gap: .75rem; align-items: center; margin-bottom: 1rem; flex-wrap: wrap; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem; }
  .card { border: 1px solid #8884; border-radius: 8px; padding: .75rem; }
  .card h2 { font-size: .9rem; margin: 0 0 .25rem; font-family: ui-monospace, monospace; }
  .card .stats { font-size: .8rem; opacity: .75; margin-top: .25rem; }
  .card img { max-width: 100%; display: block; }
  .empty { opacity: .6; }
</style>
</head>
<body>
<h1>sparkline</h1>
<div class="bar">
  <label>Window <select id="window">
    <option value="1m">1 min</option><option value="5m" selected>5 min</option>
    <option value="1h">1 hour</option><option value="">all</option>
  </select></label>
  <label>Style <select id="style"><option>line</option><option>area</option><option>bars</option></select></label>
  <span id="status" class="empty"></span>
</div>
<div id="grid" class="grid"></div>
<script>
const $ = (id) => document.getElementById(id);
const windowMs = { "1m": 60000, "5m": 300000, "1h": 3600000 };
async function refresh() {
  const w = $("window").value, style = $("style").value;
  try {
    const names = await (await fetch("/api/series")).json();
    $("status").textContent = names.length ? names.length + " series" : "no series yet - POST to /ingest";
    const grid = $("grid");
    for (const name of names) {
      let card = grid.querySelector('[data-name="' + CSS.escape(name) + '"]');
      if (!card) {
        card = document.createElement("div");
        card.className = "card";
        card.dataset.name = name;
        card.innerHTML = "<h2></h2><img alt=''><div class='stats'></div>";
        card.querySelector("h2").textContent = name;
        grid.append(card);
      }
      const q = new URLSearchParams({ style, w: 280, h: 60 });
      if (w) q.set("window", windowMs[w]);
      card.querySelector("img").src = "/spark/" + encodeURIComponent(name) + ".svg?" + q;
      const s = await (await fetch("/api/series/" + encodeURIComponent(name) + (w ? "?window=" + w : ""))).json();
      const avg = s.value == null ? "-" : Number(s.value).toPrecision(4);
      card.querySelector(".stats").textContent = "avg " + avg + " over " + s.buckets + " buckets";
    }
  } catch (e) {
    $("status").textContent = "error: " + e.message;
  }
}
$("window").onchange = $("style").onchange = refresh;
refresh();
setInterval(refresh, 2000);
</script>
</body>
</html>
`;
