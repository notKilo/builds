package main

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestRunOK(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}))
	defer srv.Close()
	if code := run([]string{"-n", "5", "-c", "2", "-json", "-H", "X-A: b", srv.URL}); code != 0 {
		t.Fatalf("exit %d", code)
	}
}

func TestRunUsageErrors(t *testing.T) {
	for _, args := range [][]string{{}, {"http://x"}, {"-n", "1", "-H", "bad", "http://x"}} {
		if code := run(args); code != 2 {
			t.Errorf("%v: exit %d, want 2", args, code)
		}
	}
}

func TestRunConnectionErrorExit1(t *testing.T) {
	if code := run([]string{"-n", "1", "-timeout", "1s", "http://127.0.0.1:1"}); code != 1 {
		t.Fatalf("exit %d, want 1", code)
	}
}
