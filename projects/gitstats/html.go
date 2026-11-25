package gitstats

import (
	"html/template"
	"io"
)

var htmlTmpl = template.Must(template.New("report").Funcs(template.FuncMap{
	"date": func(t interface{ Format(string) string }) string { return t.Format("2006-01-02") },
	"bar": func(n, max int) int {
		if max <= 0 {
			return 0
		}
		return n * 100 / max
	},
	"maxChurn": func(fs []FileStat) int {
		m := 0
		for _, f := range fs {
			m = max(m, f.Churn())
		}
		return m
	},
	"days": func() []string { return []string{"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"} },
	"maxDay": func(a [7]int) int {
		m := 0
		for _, n := range a {
			m = max(m, n)
		}
		return m
	},
}).Parse(`<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>gitstats report</title>
<style>
:root{--bg:#fff;--fg:#1c1f24;--muted:#667;--bar:#3b82f6;--line:#e2e5ea}
@media (prefers-color-scheme:dark){:root{--bg:#14161a;--fg:#e6e8eb;--muted:#9aa;--bar:#60a5fa;--line:#2a2e35}}
body{font:15px/1.5 system-ui,sans-serif;background:var(--bg);color:var(--fg);max-width:60rem;margin:0 auto;padding:1rem 1rem 3rem}
h1{margin-bottom:0}h2{margin-top:2rem;border-bottom:1px solid var(--line);padding-bottom:.25rem}
p.meta{color:var(--muted);margin-top:.25rem}
table{border-collapse:collapse;width:100%}th,td{text-align:left;padding:.3rem .5rem;border-bottom:1px solid var(--line)}
td.num{text-align:right;font-variant-numeric:tabular-nums}code{font-size:.9em;word-break:break-all}
.bar{background:var(--bar);height:.6rem;border-radius:2px;min-width:1px}
.scroll{overflow-x:auto}
</style></head><body>
<h1>gitstats report</h1>
<p class="meta">{{.Commits}} commits{{if .Commits}}, {{date .First}} to {{date .Last}}{{end}}</p>
{{if .Commits}}
<h2>Top churn</h2><div class="scroll"><table><tr><th>File</th><th>Lines</th><th></th><th>Commits</th><th>Authors</th></tr>
{{$m := maxChurn .Churn}}{{range .Churn}}<tr><td><code>{{.Path}}</code></td><td class="num">{{.Churn}}</td><td style="width:30%"><div class="bar" style="width:{{bar .Churn $m}}%"></div></td><td class="num">{{.Commits}}</td><td class="num">{{.Authors}}</td></tr>
{{end}}</table></div>
<h2>Hotspots</h2><div class="scroll"><table><tr><th>File</th><th>Score</th><th>Commits</th><th>Last change</th></tr>
{{range .Hotspots}}<tr><td><code>{{.Path}}</code></td><td class="num">{{printf "%.2f" .Score}}</td><td class="num">{{.Commits}}</td><td>{{date .Last}}</td></tr>
{{end}}</table></div>
<h2>Bus factor</h2><div class="scroll"><table><tr><th>Directory</th><th>Factor</th><th>Lines</th><th>Owners</th></tr>
{{range .BusFactor}}<tr><td><code>{{.Dir}}</code></td><td class="num">{{.Factor}}</td><td class="num">{{.Lines}}</td><td>{{range $i, $o := .Owners}}{{if $i}}, {{end}}{{$o}}{{end}}</td></tr>
{{end}}</table></div>
<h2>Authors</h2><div class="scroll"><table><tr><th>Author</th><th>Commits</th><th>Added</th><th>Deleted</th><th>Last</th></tr>
{{range .Authors}}<tr><td>{{.Name}} &lt;{{.Email}}&gt;</td><td class="num">{{.Commits}}</td><td class="num">{{.Added}}</td><td class="num">{{.Deleted}}</td><td>{{date .Last}}</td></tr>
{{end}}</table></div>
<h2>Commits by weekday</h2><table>
{{$d := days}}{{$mx := maxDay .Activity.ByWeekday}}{{range $i, $n := .Activity.ByWeekday}}<tr><td style="width:3rem">{{index $d $i}}</td><td class="num" style="width:4rem">{{$n}}</td><td><div class="bar" style="width:{{bar $n $mx}}%"></div></td></tr>
{{end}}</table>
{{end}}
</body></html>
`))

// WriteHTML writes a self-contained HTML report (no external assets).
func (r Report) WriteHTML(w io.Writer) error {
	return htmlTmpl.Execute(w, r)
}
