// Command gitstats prints churn, hotspot, bus-factor and author reports for a git repository.
package main

import (
	"flag"
	"fmt"
	"os"
	"time"

	"github.com/notkilo/builds/projects/gitstats"
)

func main() {
	if err := run(os.Args[1:], os.Stdout); err != nil {
		fmt.Fprintln(os.Stderr, "gitstats:", err)
		os.Exit(1)
	}
}

func run(args []string, out *os.File) error {
	fs := flag.NewFlagSet("gitstats", flag.ContinueOnError)
	repo := fs.String("repo", ".", "path to the git repository")
	since := fs.String("since", "", `only commits since this date (git syntax, e.g. "6 months ago")`)
	path := fs.String("path", "", "restrict to a pathspec")
	top := fs.Int("top", 10, "rows per section (0 = all)")
	depth := fs.Int("depth", 1, "directory depth for bus factor")
	halfLife := fs.Int("half-life", 90, "hotspot recency half-life in days")
	format := fs.String("format", "text", "output format: text or json")
	if err := fs.Parse(args); err != nil {
		return err
	}
	if *format != "text" && *format != "json" {
		return fmt.Errorf("unknown format %q (want text or json)", *format)
	}
	commits, err := gitstats.LoadRepo(*repo, *since, *path)
	if err != nil {
		return err
	}
	rep := gitstats.Build(commits, gitstats.Options{
		Top: *top, Depth: *depth, HalfLife: time.Duration(*halfLife) * 24 * time.Hour,
	})
	if *format == "json" {
		return rep.WriteJSON(out)
	}
	return rep.WriteText(out)
}
