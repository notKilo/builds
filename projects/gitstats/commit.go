// Package gitstats mines a git repository's history for churn, hotspots,
// ownership and activity statistics.
package gitstats

import "time"

// FileChange is one file touched by a commit.
type FileChange struct {
	Path    string // path after the commit (new name for renames)
	OldPath string // previous path when the file was renamed, else ""
	Added   int
	Deleted int
	Binary  bool
}

// Commit is a parsed commit with its file changes.
type Commit struct {
	Hash    string
	Author  string
	Email   string
	Time    time.Time
	Subject string
	Files   []FileChange
}
