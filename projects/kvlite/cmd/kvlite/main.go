// Command kvlite is a small CLI over a kvlite store directory.
package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"

	"github.com/notkilo/builds/projects/kvlite"
)

const usage = `usage: kvlite [-dir DIR] <command> [args]

commands:
  put KEY VALUE     store a value
  get KEY           print a value
  del KEY           delete a key
  scan [PREFIX]     print key=value pairs in key order
  compact           merge segments and drop dead records
  stats             show disk usage`

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

func run(args []string, out, errOut io.Writer) int {
	fs := flag.NewFlagSet("kvlite", flag.ContinueOnError)
	fs.SetOutput(errOut)
	dir := fs.String("dir", "./kvdata", "store directory")
	fs.Usage = func() { fmt.Fprintln(errOut, usage) }
	if err := fs.Parse(args); err != nil {
		return 2
	}
	rest := fs.Args()
	if len(rest) == 0 {
		fs.Usage()
		return 2
	}
	s, err := kvlite.Open(*dir, kvlite.Options{SyncEvery: true})
	if err != nil {
		fmt.Fprintln(errOut, "error:", err)
		return 1
	}
	defer s.Close()
	if err := dispatch(s, rest[0], rest[1:], out); err != nil {
		if errors.Is(err, errUsage) {
			fmt.Fprintln(errOut, usage)
			return 2
		}
		fmt.Fprintln(errOut, "error:", err)
		return 1
	}
	return 0
}

var errUsage = errors.New("usage")

func dispatch(s *kvlite.Store, cmd string, args []string, out io.Writer) error {
	need := func(n int) error {
		if len(args) != n {
			return errUsage
		}
		return nil
	}
	switch cmd {
	case "put":
		if err := need(2); err != nil {
			return err
		}
		return s.Put([]byte(args[0]), []byte(args[1]))
	case "get":
		if err := need(1); err != nil {
			return err
		}
		v, err := s.Get([]byte(args[0]))
		if err != nil {
			return err
		}
		fmt.Fprintf(out, "%s\n", v)
	case "del":
		if err := need(1); err != nil {
			return err
		}
		return s.Delete([]byte(args[0]))
	case "scan":
		var prefix []byte
		if len(args) > 1 {
			return errUsage
		}
		if len(args) == 1 {
			prefix = []byte(args[0])
		}
		return s.Scan(prefix, func(k, v []byte) bool {
			fmt.Fprintf(out, "%s=%s\n", k, v)
			return true
		})
	case "compact":
		return s.Compact()
	case "stats":
		st, err := s.Stats()
		if err != nil {
			return err
		}
		fmt.Fprintf(out, "keys: %d\nsegments: %d\ndisk bytes: %d\nlive bytes: %d\ngarbage: %.1f%%\n",
			st.Keys, st.Segments, st.DiskBytes, st.LiveBytes, st.Garbage()*100)
	default:
		return errUsage
	}
	return nil
}
