// Package runner executes the load test.
package runner

import (
	"context"
	"io"
	"net/http"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/notkilo/builds/projects/hammer/internal/stats"
)

// Config describes one load test.
type Config struct {
	URL         string
	Method      string
	Headers     http.Header
	Body        string
	Concurrency int           // number of workers
	Requests    int           // total requests; 0 means unlimited (use Duration)
	Duration    time.Duration // 0 means unlimited (use Requests)
	QPS         float64       // overall request rate cap; 0 means unlimited
	Timeout     time.Duration // per-request timeout
}

// Result is the aggregate outcome of a run.
type Result struct {
	Elapsed  time.Duration
	Latency  *stats.Recorder
	Statuses map[int]int
	Errors   map[string]int
	Total    int
}

// Run executes cfg and blocks until it finishes or ctx is cancelled.
func Run(ctx context.Context, cfg Config) *Result {
	if cfg.Concurrency < 1 {
		cfg.Concurrency = 1
	}
	if cfg.Method == "" {
		cfg.Method = http.MethodGet
	}
	if cfg.Timeout <= 0 {
		cfg.Timeout = 10 * time.Second
	}
	if cfg.Duration > 0 {
		var cancel context.CancelFunc
		ctx, cancel = context.WithTimeout(ctx, cfg.Duration)
		defer cancel()
	}

	client := &http.Client{
		Timeout:   cfg.Timeout,
		Transport: &http.Transport{MaxIdleConnsPerHost: cfg.Concurrency},
	}
	defer client.CloseIdleConnections()

	var issued atomic.Int64
	// next reports whether the caller may issue another request.
	next := func() bool {
		n := issued.Add(1)
		return cfg.Requests == 0 || n <= int64(cfg.Requests)
	}

	var tick <-chan time.Time
	if cfg.QPS > 0 {
		t := time.NewTicker(time.Duration(float64(time.Second) / cfg.QPS))
		defer t.Stop()
		tick = t.C
	}

	parts := make([]*workerResult, cfg.Concurrency)
	var wg sync.WaitGroup
	start := time.Now()
	for i := range parts {
		parts[i] = newWorkerResult()
		wg.Add(1)
		go func(w *workerResult) {
			defer wg.Done()
			for ctx.Err() == nil {
				if tick != nil {
					select {
					case <-tick:
					case <-ctx.Done():
						return
					}
				}
				if !next() {
					return
				}
				w.do(ctx, client, cfg)
			}
		}(parts[i])
	}
	wg.Wait()

	res := &Result{
		Elapsed:  time.Since(start),
		Latency:  &stats.Recorder{},
		Statuses: map[int]int{},
		Errors:   map[string]int{},
	}
	for _, p := range parts {
		res.Latency.Merge(p.lat)
		res.Total += p.total
		for k, v := range p.statuses {
			res.Statuses[k] += v
		}
		for k, v := range p.errors {
			res.Errors[k] += v
		}
	}
	return res
}

type workerResult struct {
	lat      *stats.Recorder
	statuses map[int]int
	errors   map[string]int
	total    int
}

func newWorkerResult() *workerResult {
	return &workerResult{lat: &stats.Recorder{}, statuses: map[int]int{}, errors: map[string]int{}}
}

func (w *workerResult) do(ctx context.Context, c *http.Client, cfg Config) {
	var body io.Reader
	if cfg.Body != "" {
		body = strings.NewReader(cfg.Body)
	}
	req, err := http.NewRequestWithContext(ctx, cfg.Method, cfg.URL, body)
	if err != nil {
		w.total++
		w.errors[err.Error()]++
		return
	}
	for k, vs := range cfg.Headers {
		for _, v := range vs {
			req.Header.Add(k, v)
		}
	}
	t0 := time.Now()
	resp, err := c.Do(req)
	if err != nil {
		if ctx.Err() != nil {
			return // run ended mid-request; don't count it
		}
		w.total++
		w.errors[shortErr(err)]++
		return
	}
	io.Copy(io.Discard, resp.Body)
	resp.Body.Close()
	w.total++
	w.lat.Add(time.Since(t0))
	w.statuses[resp.StatusCode]++
}

func shortErr(err error) string {
	s := err.Error()
	if len(s) > 80 {
		s = s[:80]
	}
	return s
}
