package main

import (
	"encoding/json"
	"fmt"
	"io"
	"strings"
)

// RenderOpts controls output.
type RenderOpts struct {
	ShowSecrets bool
	OnlyDiff    bool // hide rows with Status == Same
}

func display(r Row, i int, o RenderOpts) string {
	if !r.Present[i] {
		return "(missing)"
	}
	v := r.Values[i]
	if !o.ShowSecrets && IsSecretKey(r.Key) {
		return Mask(v)
	}
	if v == "" {
		return `""`
	}
	return v
}

func visible(rows []Row, o RenderOpts) []Row {
	var out []Row
	for _, r := range rows {
		if o.OnlyDiff && r.Status == Same {
			continue
		}
		out = append(out, r)
	}
	return out
}

func cells(envs []Env, rows []Row, o RenderOpts) [][]string {
	grid := [][]string{append([]string{"KEY", "STATUS"}, envNames(envs)...)}
	for _, r := range visible(rows, o) {
		line := []string{r.Key, r.Status.String()}
		for i := range envs {
			line = append(line, display(r, i, o))
		}
		grid = append(grid, line)
	}
	return grid
}

func envNames(envs []Env) []string {
	n := make([]string, len(envs))
	for i, e := range envs {
		n[i] = e.Name
	}
	return n
}

// RenderTable writes an aligned plain-text table.
func RenderTable(w io.Writer, envs []Env, rows []Row, o RenderOpts) {
	grid := cells(envs, rows, o)
	width := make([]int, len(grid[0]))
	for _, line := range grid {
		for i, c := range line {
			if len(c) > width[i] {
				width[i] = len(c)
			}
		}
	}
	for _, line := range grid {
		parts := make([]string, len(line))
		for i, c := range line {
			parts[i] = fmt.Sprintf("%-*s", width[i], c)
		}
		fmt.Fprintln(w, strings.TrimRight(strings.Join(parts, "  "), " "))
	}
}

// RenderMarkdown writes a GitHub-flavoured markdown table.
func RenderMarkdown(w io.Writer, envs []Env, rows []Row, o RenderOpts) {
	grid := cells(envs, rows, o)
	for i, line := range grid {
		esc := make([]string, len(line))
		for j, c := range line {
			esc[j] = strings.ReplaceAll(strings.ReplaceAll(c, "|", `\|`), "\n", " ")
		}
		fmt.Fprintf(w, "| %s |\n", strings.Join(esc, " | "))
		if i == 0 {
			fmt.Fprintf(w, "|%s\n", strings.Repeat(" --- |", len(line)))
		}
	}
}

// RenderJSON writes a machine-readable report. Masked values are
// replaced exactly as in the text outputs.
func RenderJSON(w io.Writer, envs []Env, rows []Row, warns []Warning, o RenderOpts) error {
	type jrow struct {
		Key    string             `json:"key"`
		Status string             `json:"status"`
		Values map[string]*string `json:"values"` // null = missing
	}
	rep := struct {
		Environments []string  `json:"environments"`
		Rows         []jrow    `json:"rows"`
		Warnings     []Warning `json:"warnings"`
	}{Environments: envNames(envs), Warnings: warns}
	if rep.Warnings == nil {
		rep.Warnings = []Warning{}
	}
	rep.Rows = []jrow{}
	for _, r := range visible(rows, o) {
		jr := jrow{Key: r.Key, Status: r.Status.String(), Values: map[string]*string{}}
		for i, e := range envs {
			if !r.Present[i] {
				jr.Values[e.Name] = nil
				continue
			}
			v := r.Values[i]
			if !o.ShowSecrets && IsSecretKey(r.Key) {
				v = Mask(v)
			}
			jr.Values[e.Name] = &v
		}
		rep.Rows = append(rep.Rows, jr)
	}
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	return enc.Encode(rep)
}
