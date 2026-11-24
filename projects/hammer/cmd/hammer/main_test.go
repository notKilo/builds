package main

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"
)

func TestRunOK(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	defer srv.Close()
	if code := run([]string{"-n", "5", "-c", "2", "-json", "-H", "X-A: b", srv.URL}, io.Discard, io.Discard); code != 0 {
		t.Fatalf("exit %d", code)
	}
}

func TestRunUsageErrors(t *testing.T) {
	for _, args := range [][]string{{}, {"http://x"}, {"-n", "1", "-H", "bad", "http://x"}} {
		if code := run(args, io.Discard, io.Discard); code != 2 {
			t.Errorf("%v: exit %d, want 2", args, code)
		}
	}
}

func TestRunConnectionErrorExit1(t *testing.T) {
	if code := run([]string{"-n", "1", "-timeout", "1s", "http://127.0.0.1:1"}, io.Discard, io.Discard); code != 1 {
		t.Fatalf("exit %d, want 1", code)
	}
}

// mixedServer returns 200 for most requests, 500 for every fifth, and is slow for every third.
func mixedServer() *httptest.Server {
	var n atomic.Int64
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		i := n.Add(1)
		if i%3 == 0 {
			time.Sleep(20 * time.Millisecond)
		}
		if i%5 == 0 {
			w.WriteHeader(http.StatusInternalServerError)
		}
	}))
}

func TestEndToEndText(t *testing.T) {
	srv := mixedServer()
	defer srv.Close()
	var out, errb bytes.Buffer
	if code := run([]string{"-n", "30", "-c", "3", srv.URL}, &out, &errb); code != 0 {
		t.Fatalf("exit %d, stderr %q", code, errb.String())
	}
	for _, want := range []string{"Requests:   30", "200: 24", "500: 6", "Latency histogram:"} {
		if !strings.Contains(out.String(), want) {
			t.Errorf("output missing %q:\n%s", want, out.String())
		}
	}
}

func TestEndToEndJSON(t *testing.T) {
	srv := mixedServer()
	defer srv.Close()
	var out bytes.Buffer
	if code := run([]string{"-n", "20", "-c", "2", "-json", "-buckets", "4", srv.URL}, &out, io.Discard); code != 0 {
		t.Fatalf("exit %d", code)
	}
	var rep struct {
		Total     int            `json:"total"`
		Statuses  map[string]int `json:"statuses"`
		Histogram []struct {
			Count int `json:"count"`
		} `json:"histogram"`
	}
	if err := json.Unmarshal(out.Bytes(), &rep); err != nil {
		t.Fatalf("invalid JSON: %v\n%s", err, out.String())
	}
	if rep.Total != 20 || rep.Statuses["200"] != 16 || rep.Statuses["500"] != 4 {
		t.Errorf("unexpected report: %+v", rep)
	}
	if len(rep.Histogram) != 4 {
		t.Errorf("got %d buckets, want 4", len(rep.Histogram))
	}
	sum := 0
	for _, b := range rep.Histogram {
		sum += b.Count
	}
	if sum != 20 {
		t.Errorf("histogram counts sum to %d, want 20", sum)
	}
}

func TestUsageGoesToStderr(t *testing.T) {
	var out, errb bytes.Buffer
	if code := run(nil, &out, &errb); code != 2 {
		t.Fatalf("exit %d, want 2", code)
	}
	if out.Len() != 0 || !strings.Contains(errb.String(), "usage: hammer") {
		t.Errorf("stdout %q stderr %q", out.String(), errb.String())
	}
}
