// Package report formats a runner.Result as text or JSON.
package report

import (
	"encoding/json"
	"fmt"
	"io"
	"sort"
	"time"

	"github.com/notkilo/builds/projects/hammer/internal/runner"
	"github.com/notkilo/builds/projects/hammer/internal/stats"
)

func ms(d time.Duration) float64 { return float64(d) / float64(time.Millisecond) }

// Text writes a human-readable report.
func Text(w io.Writer, r *runner.Result, buckets int) {
	s := r.Latency.Summarize()
	fmt.Fprintf(w, "Requests:   %d in %s (%.1f req/s)\n", r.Total, r.Elapsed.Round(time.Millisecond), rps(r))
	fmt.Fprintf(w, "Latency:    min %s  mean %s  max %s\n", s.Min.Round(time.Microsecond), s.Mean.Round(time.Microsecond), s.Max.Round(time.Microsecond))
	fmt.Fprintf(w, "Percentile: p50 %s  p90 %s  p95 %s  p99 %s\n", s.P50.Round(time.Microsecond), s.P90.Round(time.Microsecond), s.P95.Round(time.Microsecond), s.P99.Round(time.Microsecond))

	fmt.Fprintln(w, "Status codes:")
	codes := make([]int, 0, len(r.Statuses))
	for c := range r.Statuses {
		codes = append(codes, c)
	}
	sort.Ints(codes)
	for _, c := range codes {
		fmt.Fprintf(w, "  %d: %d\n", c, r.Statuses[c])
	}
	if len(r.Errors) > 0 {
		fmt.Fprintln(w, "Errors:")
		for msg, n := range r.Errors {
			fmt.Fprintf(w, "  %dx %s\n", n, msg)
		}
	}
	if h := r.Latency.Histogram(buckets); h != nil {
		fmt.Fprintln(w, "Latency histogram:")
		fmt.Fprint(w, stats.RenderHistogram(h, 40))
	}
}

func rps(r *runner.Result) float64 {
	if r.Elapsed <= 0 {
		return 0
	}
	return float64(r.Total) / r.Elapsed.Seconds()
}

type jsonBucket struct {
	LoMs  float64 `json:"lo_ms"`
	HiMs  float64 `json:"hi_ms"`
	Count int     `json:"count"`
}

type jsonReport struct {
	Total      int                `json:"total"`
	ElapsedSec float64            `json:"elapsed_sec"`
	RPS        float64            `json:"rps"`
	Latency    map[string]float64 `json:"latency_ms"`
	Statuses   map[int]int        `json:"statuses"`
	Errors     map[string]int     `json:"errors"`
	Histogram  []jsonBucket       `json:"histogram"`
}

// JSON writes a machine-readable report.
func JSON(w io.Writer, r *runner.Result, buckets int) error {
	s := r.Latency.Summarize()
	out := jsonReport{
		Total:      r.Total,
		ElapsedSec: r.Elapsed.Seconds(),
		RPS:        rps(r),
		Latency: map[string]float64{
			"min": ms(s.Min), "mean": ms(s.Mean), "max": ms(s.Max),
			"p50": ms(s.P50), "p90": ms(s.P90), "p95": ms(s.P95), "p99": ms(s.P99),
		},
		Statuses:  r.Statuses,
		Errors:    r.Errors,
		Histogram: []jsonBucket{},
	}
	for _, b := range r.Latency.Histogram(buckets) {
		out.Histogram = append(out.Histogram, jsonBucket{ms(b.Lo), ms(b.Hi), b.Count})
	}
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	return enc.Encode(out)
}
