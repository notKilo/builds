package main

import (
	"context"
	"fmt"
	"os"
)

func main() {
	if len(os.Args) < 3 || os.Args[1] != "check-once" {
		fmt.Fprintln(os.Stderr, "usage: pingboard check-once <config.json>")
		os.Exit(2)
	}
	cfg, err := LoadConfig(os.Args[2])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	down := 0
	for _, c := range cfg.Checks {
		r := Run(context.Background(), c)
		state := "UP  "
		if !r.Up {
			state = "DOWN"
			down++
		}
		fmt.Printf("%s %-20s %8s %s\n", state, c.Name, r.Latency.Round(1e5), r.Error)
	}
	if down > 0 {
		os.Exit(1)
	}
}
