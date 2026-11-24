package runner

import (
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"
)

func TestRunFixedRequests(t *testing.T) {
	var hits atomic.Int64
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		hits.Add(1)
	}))
	defer srv.Close()

	res := Run(context.Background(), Config{URL: srv.URL, Concurrency: 4, Requests: 50})
	if hits.Load() != 50 || res.Total != 50 || res.Statuses[200] != 50 {
		t.Fatalf("hits=%d total=%d statuses=%v", hits.Load(), res.Total, res.Statuses)
	}
	if res.Latency.Count() != 50 {
		t.Fatalf("latency samples = %d", res.Latency.Count())
	}
}

func TestRunDuration(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	defer srv.Close()

	res := Run(context.Background(), Config{URL: srv.URL, Concurrency: 2, Duration: 200 * time.Millisecond})
	if res.Total == 0 {
		t.Fatal("expected some requests")
	}
	if res.Elapsed < 150*time.Millisecond || res.Elapsed > 2*time.Second {
		t.Fatalf("elapsed = %v", res.Elapsed)
	}
}

func TestRunQPSLimit(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	defer srv.Close()

	res := Run(context.Background(), Config{URL: srv.URL, Concurrency: 4, Duration: 500 * time.Millisecond, QPS: 20})
	if res.Total < 5 || res.Total > 15 {
		t.Fatalf("expected ~10 requests at 20qps/0.5s, got %d", res.Total)
	}
}

func TestRunMethodHeadersBody(t *testing.T) {
	var gotMethod, gotHdr, gotBody atomic.Value
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		b, _ := io.ReadAll(r.Body)
		gotMethod.Store(r.Method)
		gotHdr.Store(r.Header.Get("X-Test"))
		gotBody.Store(string(b))
		w.WriteHeader(201)
	}))
	defer srv.Close()

	res := Run(context.Background(), Config{
		URL: srv.URL, Method: "POST", Body: "hi", Requests: 1, Concurrency: 1,
		Headers: http.Header{"X-Test": {"yes"}},
	})
	if gotMethod.Load() != "POST" || gotHdr.Load() != "yes" || gotBody.Load() != "hi" {
		t.Fatalf("request not as configured: %v %v %v", gotMethod.Load(), gotHdr.Load(), gotBody.Load())
	}
	if res.Statuses[201] != 1 {
		t.Fatalf("statuses = %v", res.Statuses)
	}
}

func TestRunCountsErrors(t *testing.T) {
	res := Run(context.Background(), Config{URL: "http://127.0.0.1:1", Requests: 3, Concurrency: 1, Timeout: time.Second})
	if res.Total != 3 || len(res.Errors) == 0 {
		t.Fatalf("total=%d errors=%v", res.Total, res.Errors)
	}
}
