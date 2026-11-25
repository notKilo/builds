package main

import (
	"bytes"
	"strings"
	"testing"
)

func kv(t *testing.T, dir string, args ...string) (string, int) {
	t.Helper()
	var out, errOut bytes.Buffer
	code := run(append([]string{"-dir", dir}, args...), &out, &errOut)
	return out.String(), code
}

func TestCLIFlow(t *testing.T) {
	dir := t.TempDir()
	for _, a := range [][]string{{"put", "b", "2"}, {"put", "a", "1"}, {"put", "a", "one"}} {
		if _, code := kv(t, dir, a...); code != 0 {
			t.Fatalf("%v exited %d", a, code)
		}
	}
	if out, _ := kv(t, dir, "get", "a"); out != "one\n" {
		t.Errorf("get = %q", out)
	}
	if out, _ := kv(t, dir, "scan"); out != "a=one\nb=2\n" {
		t.Errorf("scan = %q", out)
	}
	kv(t, dir, "del", "b")
	if _, code := kv(t, dir, "get", "b"); code != 1 {
		t.Errorf("get of deleted key exited %d, want 1", code)
	}
	if _, code := kv(t, dir, "compact"); code != 0 {
		t.Errorf("compact exited %d", code)
	}
	out, _ := kv(t, dir, "stats")
	if !strings.Contains(out, "keys: 1") {
		t.Errorf("stats = %q", out)
	}
}

func TestCLIUsageErrors(t *testing.T) {
	dir := t.TempDir()
	for _, a := range [][]string{{}, {"put", "k"}, {"bogus"}} {
		if _, code := kv(t, dir, a...); code != 2 {
			t.Errorf("%v exited %d, want 2", a, code)
		}
	}
}
