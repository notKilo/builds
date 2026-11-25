package gitstats

import (
	"strings"
	"testing"
)

func TestParseRename(t *testing.T) {
	cases := []struct{ in, newP, oldP string }{
		{"a.go", "a.go", ""},
		{"old.go => new.go", "new.go", "old.go"},
		{"src/{old => new}/f.go", "src/new/f.go", "src/old/f.go"},
		{"src/{ => sub}/f.go", "src/sub/f.go", "src/f.go"},
		{"src/{sub => }/f.go", "src/f.go", "src/sub/f.go"},
		{"{a => b}/x", "b/x", "a/x"},
	}
	for _, c := range cases {
		n, o := parseRename(c.in)
		if n != c.newP || o != c.oldP {
			t.Errorf("parseRename(%q) = %q,%q want %q,%q", c.in, n, o, c.newP, c.oldP)
		}
	}
}

func TestParseLog(t *testing.T) {
	in := "\x1eaaa\x1fAnn\x1fann@x.io\x1f1700000000\x1fAdd things\n\n" +
		"3\t1\tmain.go\n-\t-\timg.png\n2\t0\told.go => new.go\n" +
		"\x1ebbb\x1fBob\x1fbob@x.io\x1f1690000000\x1fsubject\x1fwith sep\n\n"
	cs, err := ParseLog(strings.NewReader(in))
	if err != nil {
		t.Fatal(err)
	}
	if len(cs) != 2 {
		t.Fatalf("got %d commits", len(cs))
	}
	c := cs[0]
	if c.Hash != "aaa" || c.Author != "Ann" || c.Time.Unix() != 1700000000 || len(c.Files) != 3 {
		t.Fatalf("bad commit %+v", c)
	}
	if !c.Files[1].Binary || c.Files[2].OldPath != "old.go" || c.Files[0].Added != 3 {
		t.Fatalf("bad files %+v", c.Files)
	}
	if cs[1].Subject != "subject\x1fwith sep" || len(cs[1].Files) != 0 {
		t.Fatalf("bad second commit %+v", cs[1])
	}
}

func TestParseLogErrors(t *testing.T) {
	for _, in := range []string{"\x1eonlyhash\n", "\x1ea\x1fb\x1fc\x1fnotnum\x1fs\n", "\x1ea\x1fb\x1fc\x1f1\x1fs\n\nbadline\n"} {
		if _, err := ParseLog(strings.NewReader(in)); err == nil {
			t.Errorf("expected error for %q", in)
		}
	}
}
