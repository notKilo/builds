package main

import (
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
)

const usage = `usage: envdiff [flags] FILE FILE [FILE...]

Compare config files (.env, export-style, .json) across environments.

flags:
`

// Exit codes: 0 clean, 1 differences or warnings found, 2 usage/IO error.
func main() { os.Exit(run(os.Args[1:], os.Stdout, os.Stderr)) }

func run(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("envdiff", flag.ContinueOnError)
	fs.SetOutput(stderr)
	format := fs.String("format", "table", "output format: table, markdown, json")
	showSecrets := fs.Bool("show-secrets", false, "print secret values instead of masking them")
	onlyDiff := fs.Bool("only-diff", false, "hide keys that are identical everywhere")
	example := fs.String("example", "", "check each file against this template (.env.example) instead of comparing")
	ignore := fs.String("ignore", "", "comma-separated keys to ignore")
	fs.Usage = func() { fmt.Fprint(stderr, usage); fs.PrintDefaults() }
	if err := fs.Parse(args); err != nil {
		return 2
	}
	files := fs.Args()
	if *example == "" && len(files) < 2 || *example != "" && len(files) < 1 {
		fs.Usage()
		return 2
	}

	envs, err := loadAll(files, *ignore)
	if err != nil {
		fmt.Fprintln(stderr, "envdiff:", err)
		return 2
	}
	if *example != "" {
		return runExample(*example, envs, *ignore, stdout, stderr)
	}

	opts := RenderOpts{ShowSecrets: *showSecrets, OnlyDiff: *onlyDiff}
	rows := Compare(envs)
	warns := Lint(envs, rows)
	switch *format {
	case "table":
		RenderTable(stdout, envs, rows, opts)
	case "markdown":
		RenderMarkdown(stdout, envs, rows, opts)
	case "json":
		if err := RenderJSON(stdout, envs, rows, warns, opts); err != nil {
			fmt.Fprintln(stderr, "envdiff:", err)
			return 2
		}
	default:
		fmt.Fprintf(stderr, "envdiff: unknown format %q\n", *format)
		return 2
	}
	if *format != "json" {
		for _, w := range warns {
			fmt.Fprintf(stdout, "warning: %s in %s: %s\n", w.Key, w.Env, w.Reason)
		}
	}
	for _, r := range rows {
		if r.Status != Same {
			return 1
		}
	}
	if len(warns) > 0 {
		return 1
	}
	return 0
}

func loadAll(files []string, ignore string) ([]Env, error) {
	skip := map[string]bool{}
	for _, k := range strings.Split(ignore, ",") {
		if k = strings.TrimSpace(k); k != "" {
			skip[k] = true
		}
	}
	var envs []Env
	for _, f := range files {
		e, err := LoadFile(f)
		if err != nil {
			return nil, err
		}
		for k := range skip {
			delete(e.Vars, k)
		}
		envs = append(envs, e)
	}
	return envs, nil
}

// runExample checks that every env defines exactly the template's keys.
func runExample(path string, envs []Env, ignore string, stdout, stderr io.Writer) int {
	tmpl, err := loadAll([]string{path}, ignore)
	if err != nil {
		fmt.Fprintln(stderr, "envdiff:", err)
		return 2
	}
	problems := 0
	for _, e := range envs {
		missing, extra := CheckAgainst(tmpl[0], e)
		for _, k := range missing {
			fmt.Fprintf(stdout, "%s: missing %s\n", e.Name, k)
		}
		for _, k := range extra {
			fmt.Fprintf(stdout, "%s: undocumented %s (not in %s)\n", e.Name, k, tmpl[0].Name)
		}
		problems += len(missing) + len(extra)
	}
	if problems > 0 {
		return 1
	}
	fmt.Fprintln(stdout, "ok: all files match the template")
	return 0
}
