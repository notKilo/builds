package kvlite

import (
	"bufio"
	"io"
)

// newSeqReader returns a buffered sequential reader over f from offset 0.
func newSeqReader(f io.ReadSeeker) io.Reader {
	f.Seek(0, io.SeekStart)
	return bufio.NewReaderSize(f, 64<<10)
}
