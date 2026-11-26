package main

import "testing"

func TestParseDotenv(t *testing.T) {
	src := "# comment\nA=1\nexport B = two # trailing\nC=\"x\\ny\"\nD='raw $x'\nE=\nF=\"multi\nline\"\r\nG=a=b\n"
	got, err := ParseDotenv(src)
	if err != nil {
		t.Fatal(err)
	}
	want := map[string]string{"A": "1", "B": "two", "C": "x\ny", "D": "raw $x", "E": "", "F": "multi\nline", "G": "a=b"}
	if len(got) != len(want) {
		t.Fatalf("got %v", got)
	}
	for k, v := range want {
		if got[k] != v {
			t.Errorf("%s = %q, want %q", k, got[k], v)
		}
	}
}

func TestParseDotenvErrors(t *testing.T) {
	for _, src := range []string{"novalue\n", "1BAD=x\n", "A='open\n", "A=\"open\n", "=x\n"} {
		if _, err := ParseDotenv(src); err == nil {
			t.Errorf("expected error for %q", src)
		}
	}
}
