package main

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func write(t *testing.T, dir, name, body string) string {
	t.Helper()
	p := filepath.Join(dir, name)
	if err := os.WriteFile(p, []byte(body), 0o644); err != nil {
		t.Fatal(err)
	}
	return p
}

func TestRunExitCodes(t *testing.T) {
	d := t.TempDir()
	a := write(t, d, "a.env", "HOST=x\nTOKEN=t1\n")
	same := write(t, d, "b.env", "HOST=x\nTOKEN=t1\n")
	diff := write(t, d, "c.env", "HOST=y\nTOKEN=t1\n")
	var out, errb bytes.Buffer
	if c := run([]string{a, same}, &out, &errb); c != 0 {
		t.Errorf("identical: code %d\n%s%s", c, out.String(), errb.String())
	}
	out.Reset()
	if c := run([]string{a, diff}, &out, &errb); c != 1 || !strings.Contains(out.String(), "differs") {
		t.Errorf("differ: code %d\n%s", c, out.String())
	}
	if c := run([]string{a}, &out, &errb); c != 2 {
		t.Errorf("one file: code %d", c)
	}
	if c := run([]string{a, filepath.Join(d, "none")}, &out, &errb); c != 2 {
		t.Errorf("missing file: code %d", c)
	}
	if c := run([]string{"-format", "yaml", a, same}, &out, &errb); c != 2 {
		t.Errorf("bad format: code %d", c)
	}
}

func TestRunIgnoreAndWarnings(t *testing.T) {
	d := t.TempDir()
	a := write(t, d, "a.env", "HOST=x\nNOISE=1\nPASS=\n")
	b := write(t, d, "b.env", "HOST=x\nNOISE=2\nPASS=\n")
	var out, errb bytes.Buffer
	c := run([]string{"-ignore", "NOISE", a, b}, &out, &errb)
	if c != 1 || !strings.Contains(out.String(), "warning: PASS in a.env: empty value") {
		t.Errorf("code %d\n%s", c, out.String())
	}
	if strings.Contains(out.String(), "NOISE") {
		t.Error("ignored key shown")
	}
}

func TestRunExampleMode(t *testing.T) {
	d := t.TempDir()
	ex := write(t, d, ".env.example", "A=\nB=\n")
	good := write(t, d, "good.env", "A=1\nB=2\n")
	bad := write(t, d, "bad.env", "A=1\nC=3\n")
	var out, errb bytes.Buffer
	if c := run([]string{"-example", ex, good}, &out, &errb); c != 0 {
		t.Errorf("good: %d %s", c, out.String())
	}
	out.Reset()
	c := run([]string{"-example", ex, bad}, &out, &errb)
	if c != 1 || !strings.Contains(out.String(), "bad.env: missing B") || !strings.Contains(out.String(), "undocumented C") {
		t.Errorf("bad: %d\n%s", c, out.String())
	}
}
