package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestParseJSONFlatten(t *testing.T) {
	got, err := ParseJSON([]byte(`{"db":{"host":"h","port":5432,"opts":["a","b"]},"debug":true,"x":null,"r":1.5}`))
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]string{"db.host": "h", "db.port": "5432", "db.opts.0": "a", "db.opts.1": "b", "debug": "true", "x": "", "r": "1.5"}
	for k, v := range want {
		if got[k] != v {
			t.Errorf("%s = %q, want %q", k, got[k], v)
		}
	}
	if len(got) != len(want) {
		t.Errorf("extra keys: %v", got)
	}
	if _, err := ParseJSON([]byte(`[1]`)); err == nil {
		t.Error("expected error for non-object")
	}
}

func TestLoadFile(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "prod.env")
	os.WriteFile(p, []byte("A=1\n"), 0o644)
	e, err := LoadFile(p)
	if err != nil || e.Name != "prod.env" || e.Vars["A"] != "1" {
		t.Fatalf("%+v %v", e, err)
	}
	j := filepath.Join(dir, "c.json")
	os.WriteFile(j, []byte(`{"a":{"b":"c"}}`), 0o644)
	e, err = LoadFile(j)
	if err != nil || e.Vars["a.b"] != "c" {
		t.Fatalf("%+v %v", e, err)
	}
	if _, err := LoadFile(filepath.Join(dir, "nope")); err == nil {
		t.Error("expected error")
	}
}

func TestCompare(t *testing.T) {
	envs := []Env{
		{"dev", map[string]string{"A": "1", "B": "x", "C": "1"}},
		{"prod", map[string]string{"A": "1", "B": "y", "D": ""}},
	}
	rows := Compare(envs)
	want := map[string]Status{"A": Same, "B": Differs, "C": Missing, "D": Missing}
	if len(rows) != 4 {
		t.Fatalf("rows = %d", len(rows))
	}
	for i, r := range rows {
		if r.Status != want[r.Key] {
			t.Errorf("%s: %v, want %v", r.Key, r.Status, want[r.Key])
		}
		if i > 0 && rows[i-1].Key >= r.Key {
			t.Error("not sorted")
		}
	}
	if rows[2].Present[1] || !rows[2].Present[0] {
		t.Error("presence wrong for C")
	}
}
