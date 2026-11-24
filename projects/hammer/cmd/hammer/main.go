// Command hammer is a small HTTP load tester.
package main

import (
	"context"
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"time"

	"github.com/notkilo/builds/projects/hammer/internal/report"
	"github.com/notkilo/builds/projects/hammer/internal/runner"
)

type headerFlags []string

func (h *headerFlags) String() string     { return strings.Join(*h, ", ") }
func (h *headerFlags) Set(v string) error { *h = append(*h, v); return nil }

func main() { os.Exit(run(os.Args[1:], os.Stdout, os.Stderr)) }

func run(args []string, stdout, stderr io.Writer) int {
	fs := flag.NewFlagSet("hammer", flag.ContinueOnError)
	var headers headerFlags
	cfg := runner.Config{}
	fs.IntVar(&cfg.Concurrency, "c", 10, "number of concurrent workers")
	fs.IntVar(&cfg.Requests, "n", 0, "total number of requests (0 = unlimited)")
	fs.DurationVar(&cfg.Duration, "d", 0, "test duration, e.g. 10s (0 = unlimited)")
	fs.Float64Var(&cfg.QPS, "qps", 0, "overall requests per second cap (0 = unlimited)")
	fs.DurationVar(&cfg.Timeout, "timeout", 10*time.Second, "per-request timeout")
	fs.StringVar(&cfg.Method, "X", "GET", "HTTP method")
	fs.StringVar(&cfg.Body, "body", "", "request body")
	fs.Var(&headers, "H", "request header 'Name: value' (repeatable)")
	asJSON := fs.Bool("json", false, "print the report as JSON")
	buckets := fs.Int("buckets", 10, "histogram buckets")
	fs.SetOutput(stderr)
	fs.Usage = func() {
		fmt.Fprintln(fs.Output(), "usage: hammer [flags] URL")
		fs.PrintDefaults()
	}
	if err := fs.Parse(args); err != nil {
		return 2
	}
	if fs.NArg() != 1 {
		fs.Usage()
		return 2
	}
	cfg.URL = fs.Arg(0)
	if cfg.Requests == 0 && cfg.Duration == 0 {
		fmt.Fprintln(stderr, "hammer: set -n or -d")
		return 2
	}
	cfg.Headers = http.Header{}
	for _, h := range headers {
		name, val, ok := strings.Cut(h, ":")
		if !ok {
			fmt.Fprintf(stderr, "hammer: bad header %q\n", h)
			return 2
		}
		cfg.Headers.Add(strings.TrimSpace(name), strings.TrimSpace(val))
	}

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	res := runner.Run(ctx, cfg)

	if *asJSON {
		if err := report.JSON(stdout, res, *buckets); err != nil {
			fmt.Fprintln(stderr, "hammer:", err)
			return 1
		}
	} else {
		report.Text(stdout, res, *buckets)
	}
	if len(res.Errors) > 0 {
		return 1
	}
	return 0
}
