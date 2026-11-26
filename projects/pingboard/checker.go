package main

import (
	"context"
	"fmt"
	"io"
	"net"
	"net/http"
	"strings"
	"time"
)

// Result is the outcome of a single check run.
type Result struct {
	Check   string
	Time    time.Time
	Up      bool
	Latency time.Duration
	Status  int // HTTP status, 0 for tcp
	Error   string
}

// Run executes a check once.
func Run(ctx context.Context, c Check) Result {
	ctx, cancel := context.WithTimeout(ctx, c.Timeout.D())
	defer cancel()
	start := time.Now()
	r := Result{Check: c.Name, Time: start}
	switch c.Type {
	case "http":
		runHTTP(ctx, c, &r)
	case "tcp":
		runTCP(ctx, c, &r)
	}
	r.Latency = time.Since(start)
	return r
}

func runHTTP(ctx context.Context, c Check, r *Result) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.Target, nil)
	if err != nil {
		r.Error = err.Error()
		return
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		r.Error = err.Error()
		return
	}
	defer resp.Body.Close()
	r.Status = resp.StatusCode
	want := c.ExpectStatus
	if want == 0 {
		if resp.StatusCode < 200 || resp.StatusCode > 299 {
			r.Error = fmt.Sprintf("unexpected status %d", resp.StatusCode)
			return
		}
	} else if resp.StatusCode != want {
		r.Error = fmt.Sprintf("status %d, want %d", resp.StatusCode, want)
		return
	}
	if c.ExpectBody != "" {
		body, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
		if err != nil {
			r.Error = err.Error()
			return
		}
		if !strings.Contains(string(body), c.ExpectBody) {
			r.Error = fmt.Sprintf("body does not contain %q", c.ExpectBody)
			return
		}
	}
	r.Up = true
}

func runTCP(ctx context.Context, c Check, r *Result) {
	var d net.Dialer
	conn, err := d.DialContext(ctx, "tcp", c.Target)
	if err != nil {
		r.Error = err.Error()
		return
	}
	conn.Close()
	r.Up = true
}
