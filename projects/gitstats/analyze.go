package gitstats

import (
	"math"
	"path"
	"sort"
	"strings"
	"time"
)

// FileStat aggregates the history of one file (following renames).
type FileStat struct {
	Path    string    `json:"path"`
	Commits int       `json:"commits"`
	Added   int       `json:"added"`
	Deleted int       `json:"deleted"`
	Authors int       `json:"authors"`
	Last    time.Time `json:"last_change"`
	Score   float64   `json:"hotspot_score"`
}

// Churn is added+deleted lines.
func (f FileStat) Churn() int { return f.Added + f.Deleted }

// resolver maps historical paths to the file's current name. Commits are
// supplied newest first, so when a rename new<-old is seen, all older
// entries for old belong to new.
type resolver map[string]string

func (r resolver) resolve(p string) string {
	if cur, ok := r[p]; ok {
		return cur
	}
	return p
}

func (r resolver) note(fc FileChange) {
	if fc.OldPath != "" {
		r[fc.OldPath] = r.resolve(fc.Path)
	}
}

// Files aggregates per-file statistics. now anchors the recency weighting
// used for the hotspot score: each change counts 0.5^(age/halfLife).
func Files(commits []Commit, now time.Time, halfLife time.Duration) []FileStat {
	res := resolver{}
	type acc struct {
		FileStat
		authors map[string]bool
	}
	byPath := map[string]*acc{}
	for _, c := range commits {
		for _, fc := range c.Files {
			cur := res.resolve(fc.Path)
			res.note(fc)
			a := byPath[cur]
			if a == nil {
				a = &acc{FileStat: FileStat{Path: cur}, authors: map[string]bool{}}
				byPath[cur] = a
			}
			a.Commits++
			a.Added += fc.Added
			a.Deleted += fc.Deleted
			a.authors[c.Email] = true
			if c.Time.After(a.Last) {
				a.Last = c.Time
			}
			a.Score += decay(now.Sub(c.Time), halfLife) * math.Log1p(float64(fc.Added+fc.Deleted))
		}
	}
	out := make([]FileStat, 0, len(byPath))
	for _, a := range byPath {
		a.Authors = len(a.authors)
		out = append(out, a.FileStat)
	}
	return out
}

func decay(age, halfLife time.Duration) float64 {
	if age < 0 {
		age = 0
	}
	if halfLife <= 0 {
		return 1
	}
	return math.Pow(0.5, float64(age)/float64(halfLife))
}

// TopByChurn returns up to n files with the most changed lines.
func TopByChurn(fs []FileStat, n int) []FileStat {
	return top(fs, n, func(a, b FileStat) bool {
		if a.Churn() != b.Churn() {
			return a.Churn() > b.Churn()
		}
		return a.Path < b.Path
	})
}

// TopByHotspot returns up to n files with the highest recency-weighted score.
func TopByHotspot(fs []FileStat, n int) []FileStat {
	return top(fs, n, func(a, b FileStat) bool {
		if a.Score != b.Score {
			return a.Score > b.Score
		}
		return a.Path < b.Path
	})
}

func top(fs []FileStat, n int, less func(a, b FileStat) bool) []FileStat {
	out := append([]FileStat(nil), fs...)
	sort.Slice(out, func(i, j int) bool { return less(out[i], out[j]) })
	if n > 0 && len(out) > n {
		out = out[:n]
	}
	return out
}

// DirChurn sums churn per directory at the given depth (1 = top level).
// Files in the repo root are grouped under ".".
func DirChurn(fs []FileStat, depth int) map[string]int {
	out := map[string]int{}
	for _, f := range fs {
		out[dirAt(f.Path, depth)] += f.Churn()
	}
	return out
}

func dirAt(p string, depth int) string {
	parts := strings.Split(path.Dir(p), "/")
	if parts[0] == "." {
		return "."
	}
	if depth < len(parts) {
		parts = parts[:depth]
	}
	return strings.Join(parts, "/")
}
