package gitstats

import (
	"bytes"
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func gitIn(t *testing.T, dir string, env []string, args ...string) {
	t.Helper()
	cmd := exec.Command("git", append([]string{"-C", dir}, args...)...)
	cmd.Env = append(os.Environ(), env...)
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("git %v: %v\n%s", args, err, out)
	}
}

func TestLoadRepoEndToEnd(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("git not installed")
	}
	dir := t.TempDir()
	gitIn(t, dir, nil, "init", "-q")
	write := func(name, body string) {
		p := filepath.Join(dir, name)
		os.MkdirAll(filepath.Dir(p), 0o755)
		if err := os.WriteFile(p, []byte(body), 0o644); err != nil {
			t.Fatal(err)
		}
	}
	commitAs := func(who, date, msg string) {
		env := []string{
			"GIT_AUTHOR_NAME=" + who, "GIT_AUTHOR_EMAIL=" + who + "@x.io", "GIT_AUTHOR_DATE=" + date,
			"GIT_COMMITTER_NAME=" + who, "GIT_COMMITTER_EMAIL=" + who + "@x.io", "GIT_COMMITTER_DATE=" + date,
		}
		gitIn(t, dir, nil, "add", "-A")
		gitIn(t, dir, env, "-c", "commit.gpgsign=false", "commit", "-q", "-m", msg)
	}
	write("src/a.go", strings.Repeat("line\n", 20))
	commitAs("ann", "2026-01-01T10:00:00Z", "first")
	write("src/a.go", strings.Repeat("line\n", 20)+"more\n")
	write("README.md", "hi\n")
	commitAs("bob", "2026-01-02T10:00:00Z", "second")
	gitIn(t, dir, nil, "mv", "src/a.go", "src/b.go")
	commitAs("ann", "2026-01-03T10:00:00Z", "rename")

	commits, err := LoadRepo(dir, "", "")
	if err != nil {
		t.Fatal(err)
	}
	if len(commits) != 3 || commits[0].Subject != "rename" {
		t.Fatalf("commits: %+v", commits)
	}
	rep := Build(commits, Options{Now: time.Date(2026, 1, 4, 0, 0, 0, 0, time.UTC), Top: 5})
	var b *FileStat
	for i := range rep.Churn {
		if rep.Churn[i].Path == "src/b.go" {
			b = &rep.Churn[i]
		}
	}
	if b == nil || b.Commits != 3 || b.Added != 21 {
		t.Fatalf("rename not followed: %+v", rep.Churn)
	}
	var buf bytes.Buffer
	if err := rep.WriteText(&buf); err != nil || !strings.Contains(buf.String(), "src/b.go") {
		t.Fatalf("text output: %v\n%s", err, buf.String())
	}
	buf.Reset()
	if err := rep.WriteJSON(&buf); err != nil {
		t.Fatal(err)
	}
	var back Report
	if err := json.Unmarshal(buf.Bytes(), &back); err != nil || back.Commits != 3 {
		t.Fatalf("json roundtrip: %v", err)
	}
}

func TestLoadRepoNotARepo(t *testing.T) {
	if _, err := LoadRepo(t.TempDir(), "", ""); err == nil {
		t.Fatal("expected error")
	}
}

func TestEmptyReport(t *testing.T) {
	var buf bytes.Buffer
	Build(nil, Options{}).WriteText(&buf)
	if !strings.Contains(buf.String(), "no commits") {
		t.Fatal(buf.String())
	}
}
