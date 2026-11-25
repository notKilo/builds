package gitstats

import (
	"testing"
	"time"
)

var t0 = time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)

func commit(email string, daysAgo int, files ...FileChange) Commit {
	return Commit{Hash: email, Author: email, Email: email, Time: t0.AddDate(0, 0, -daysAgo), Files: files}
}

func TestFilesFollowsRenames(t *testing.T) {
	cs := []Commit{ // newest first
		commit("a", 1, FileChange{Path: "b.go", Added: 1}),
		commit("a", 2, FileChange{Path: "b.go", OldPath: "a.go"}),
		commit("b", 3, FileChange{Path: "a.go", Added: 10, Deleted: 2}),
	}
	fs := Files(cs, t0, 30*24*time.Hour)
	if len(fs) != 1 {
		t.Fatalf("want 1 file, got %+v", fs)
	}
	f := fs[0]
	if f.Path != "b.go" || f.Commits != 3 || f.Added != 11 || f.Deleted != 2 || f.Authors != 2 {
		t.Fatalf("bad stat %+v", f)
	}
	if !f.Last.Equal(t0.AddDate(0, 0, -1)) {
		t.Fatalf("bad last %v", f.Last)
	}
}

func TestHotspotPrefersRecent(t *testing.T) {
	cs := []Commit{
		commit("a", 1, FileChange{Path: "new.go", Added: 20}),
		commit("a", 400, FileChange{Path: "old.go", Added: 20}),
	}
	fs := Files(cs, t0, 30*24*time.Hour)
	top := TopByHotspot(fs, 1)
	if top[0].Path != "new.go" {
		t.Fatalf("got %v", top[0].Path)
	}
	if got := TopByChurn(fs, 0); got[0].Path != "new.go" { // tie broken by path
		t.Fatalf("tie-break: %v", got[0].Path)
	}
}

func TestDirChurn(t *testing.T) {
	fs := []FileStat{
		{Path: "main.go", Added: 1},
		{Path: "pkg/a/x.go", Added: 2, Deleted: 1},
		{Path: "pkg/b/y.go", Added: 4},
	}
	d := DirChurn(fs, 1)
	if d["."] != 1 || d["pkg"] != 7 {
		t.Fatalf("depth1 %v", d)
	}
	d = DirChurn(fs, 2)
	if d["pkg/a"] != 3 || d["pkg/b"] != 4 {
		t.Fatalf("depth2 %v", d)
	}
}
