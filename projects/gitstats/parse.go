package gitstats

import (
	"fmt"
	"io"
	"os/exec"
	"strconv"
	"strings"
	"time"
)

const (
	recordSep = "\x1e"
	fieldSep  = "\x1f"
	// LogFormat is the --format string the parser expects.
	logFormat = "%x1e%H%x1f%an%x1f%ae%x1f%at%x1f%s"
)

// ParseLog parses the output of
// `git log --numstat -M --format=<logFormat>` into commits (newest first,
// as git emits them).
func ParseLog(r io.Reader) ([]Commit, error) {
	data, err := io.ReadAll(r)
	if err != nil {
		return nil, err
	}
	var commits []Commit
	for _, rec := range strings.Split(string(data), recordSep) {
		if strings.TrimSpace(rec) == "" {
			continue
		}
		c, err := parseRecord(rec)
		if err != nil {
			return nil, err
		}
		commits = append(commits, c)
	}
	return commits, nil
}

func parseRecord(rec string) (Commit, error) {
	lines := strings.Split(rec, "\n")
	head := strings.Split(lines[0], fieldSep)
	if len(head) < 5 {
		return Commit{}, fmt.Errorf("malformed commit header %q", lines[0])
	}
	ts, err := strconv.ParseInt(head[3], 10, 64)
	if err != nil {
		return Commit{}, fmt.Errorf("bad timestamp %q: %w", head[3], err)
	}
	c := Commit{
		Hash:    head[0],
		Author:  head[1],
		Email:   head[2],
		Time:    time.Unix(ts, 0).UTC(),
		Subject: strings.Join(head[4:], fieldSep),
	}
	for _, line := range lines[1:] {
		if strings.TrimSpace(line) == "" {
			continue
		}
		fc, err := parseNumstat(line)
		if err != nil {
			return Commit{}, err
		}
		c.Files = append(c.Files, fc)
	}
	return c, nil
}

func parseNumstat(line string) (FileChange, error) {
	parts := strings.SplitN(line, "\t", 3)
	if len(parts) != 3 {
		return FileChange{}, fmt.Errorf("malformed numstat line %q", line)
	}
	fc := FileChange{}
	fc.Path, fc.OldPath = parseRename(parts[2])
	if parts[0] == "-" && parts[1] == "-" {
		fc.Binary = true
		return fc, nil
	}
	var err error
	if fc.Added, err = strconv.Atoi(parts[0]); err != nil {
		return FileChange{}, fmt.Errorf("bad added count in %q", line)
	}
	if fc.Deleted, err = strconv.Atoi(parts[1]); err != nil {
		return FileChange{}, fmt.Errorf("bad deleted count in %q", line)
	}
	return fc, nil
}

// parseRename handles git's rename notations: "old => new" and
// "dir/{old => new}/file".
func parseRename(s string) (newPath, oldPath string) {
	open := strings.Index(s, "{")
	arrow := strings.Index(s, " => ")
	if arrow < 0 {
		return s, ""
	}
	if open < 0 || open > arrow {
		return s[arrow+4:], s[:arrow]
	}
	closeIdx := strings.Index(s, "}")
	if closeIdx < arrow {
		return s, ""
	}
	prefix, suffix := s[:open], s[closeIdx+1:]
	from, to := s[open+1:arrow], s[arrow+4:closeIdx]
	join := func(mid string) string {
		p := prefix + mid + suffix
		return strings.ReplaceAll(p, "//", "/") // empty mid leaves a double slash
	}
	return join(to), join(from)
}

// LoadRepo runs git in dir and returns its history. since and path are
// optional filters (path is a pathspec).
func LoadRepo(dir, since, path string) ([]Commit, error) {
	args := []string{"-C", dir, "log", "--numstat", "-M", "--no-color", "--format=" + logFormat}
	if since != "" {
		args = append(args, "--since="+since)
	}
	if path != "" {
		args = append(args, "--", path)
	}
	cmd := exec.Command("git", args...)
	out, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}
	var stderr strings.Builder
	cmd.Stderr = &stderr
	if err := cmd.Start(); err != nil {
		return nil, err
	}
	commits, perr := ParseLog(out)
	if err := cmd.Wait(); err != nil {
		return nil, fmt.Errorf("git log failed: %s", strings.TrimSpace(stderr.String()))
	}
	return commits, perr
}
