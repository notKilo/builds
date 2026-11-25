// Package kvlite is a small embeddable key-value store. Writes are appended to
// checksummed log segments and an in-memory index maps each live key to the
// position of its latest record.
package kvlite

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
)

var (
	ErrNotFound  = errors.New("kvlite: key not found")
	ErrClosed    = errors.New("kvlite: store is closed")
	ErrKeyTooBig = errors.New("kvlite: key too large")
	ErrValTooBig = errors.New("kvlite: value too large")
	ErrEmptyKey  = errors.New("kvlite: empty key")
)

// Options configure a Store. The zero value is usable.
type Options struct {
	// MaxSegmentSize is the size after which the active segment is rolled.
	// Zero means DefaultMaxSegmentSize.
	MaxSegmentSize int64
	// SyncEvery makes every write fsync before returning.
	SyncEvery bool
}

const DefaultMaxSegmentSize = 64 << 20

type entry struct {
	seg  uint32
	off  int64
	size int
}

// Store is safe for concurrent use.
type Store struct {
	mu     sync.RWMutex
	dir    string
	opts   Options
	index  map[string]entry
	files  map[uint32]*os.File // every segment, read access
	active *os.File
	actID  uint32
	actLen int64
	closed bool
}

func segmentName(id uint32) string { return fmt.Sprintf("%08d.log", id) }

func listSegments(dir string) ([]uint32, error) {
	ents, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	var ids []uint32
	for _, e := range ents {
		var id uint32
		if !strings.HasSuffix(e.Name(), ".log") {
			continue
		}
		if _, err := fmt.Sscanf(e.Name(), "%08d.log", &id); err == nil && e.Name() == segmentName(id) {
			ids = append(ids, id)
		}
	}
	sort.Slice(ids, func(i, j int) bool { return ids[i] < ids[j] })
	return ids, nil
}

// Open opens or creates a store in dir. A torn or corrupt tail on the newest
// segment (the signature of a crash mid-write) is truncated away; corruption
// anywhere else is reported as an error.
func Open(dir string, opts Options) (*Store, error) {
	if opts.MaxSegmentSize <= 0 {
		opts.MaxSegmentSize = DefaultMaxSegmentSize
	}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, err
	}
	s := &Store{dir: dir, opts: opts, index: map[string]entry{}, files: map[uint32]*os.File{}}
	ids, err := listSegments(dir)
	if err != nil {
		return nil, err
	}
	for i, id := range ids {
		last := i == len(ids)-1
		if err := s.loadSegment(id, last); err != nil {
			s.closeFiles()
			return nil, err
		}
	}
	if len(ids) == 0 {
		err = s.newActive(1)
	} else {
		err = s.reopenActive(ids[len(ids)-1])
	}
	if err != nil {
		s.closeFiles()
		return nil, err
	}
	return s, nil
}

// loadSegment replays one segment into the index.
func (s *Store) loadSegment(id uint32, last bool) error {
	path := filepath.Join(s.dir, segmentName(id))
	f, err := os.OpenFile(path, os.O_RDWR, 0o644)
	if err != nil {
		return err
	}
	s.files[id] = f
	var off int64
	r := newSeqReader(f)
	for {
		rec, size, err := readRecord(r)
		if err == io.EOF {
			return nil
		}
		if errors.Is(err, ErrCorrupt) {
			if !last {
				return fmt.Errorf("%s at offset %d: %w", segmentName(id), off, ErrCorrupt)
			}
			return f.Truncate(off)
		}
		if err != nil {
			return err
		}
		if rec.tombstone {
			delete(s.index, string(rec.key))
		} else {
			s.index[string(rec.key)] = entry{seg: id, off: off, size: size}
		}
		off += int64(size)
	}
}

func (s *Store) newActive(id uint32) error {
	f, err := os.OpenFile(filepath.Join(s.dir, segmentName(id)), os.O_RDWR|os.O_CREATE|os.O_EXCL, 0o644)
	if err != nil {
		return err
	}
	s.files[id] = f
	s.active, s.actID, s.actLen = f, id, 0
	return nil
}

func (s *Store) reopenActive(id uint32) error {
	f := s.files[id]
	n, err := f.Seek(0, 2)
	if err != nil {
		return err
	}
	s.active, s.actID, s.actLen = f, id, n
	return nil
}

func (s *Store) closeFiles() {
	for _, f := range s.files {
		f.Close()
	}
}

func checkKV(key, value []byte) error {
	switch {
	case len(key) == 0:
		return ErrEmptyKey
	case len(key) > MaxKeySize:
		return ErrKeyTooBig
	case len(value) > MaxValueSize:
		return ErrValTooBig
	}
	return nil
}

// Put stores value under key, replacing any previous value.
func (s *Store) Put(key, value []byte) error {
	if err := checkKV(key, value); err != nil {
		return err
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return ErrClosed
	}
	e, err := s.append(record{key: key, value: value})
	if err != nil {
		return err
	}
	s.index[string(key)] = e
	return nil
}

// Delete removes key. Deleting a missing key is not an error.
func (s *Store) Delete(key []byte) error {
	if err := checkKV(key, nil); err != nil {
		return err
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return ErrClosed
	}
	if _, ok := s.index[string(key)]; !ok {
		return nil
	}
	if _, err := s.append(record{key: key, tombstone: true}); err != nil {
		return err
	}
	delete(s.index, string(key))
	return nil
}

// append writes rec to the active segment. The caller holds s.mu.
func (s *Store) append(rec record) (entry, error) {
	buf := rec.encode()
	if _, err := s.active.WriteAt(buf, s.actLen); err != nil {
		return entry{}, err
	}
	if s.opts.SyncEvery {
		if err := s.active.Sync(); err != nil {
			return entry{}, err
		}
	}
	e := entry{seg: s.actID, off: s.actLen, size: len(buf)}
	s.actLen += int64(len(buf))
	return e, nil
}

// Get returns a copy of the value stored under key.
func (s *Store) Get(key []byte) ([]byte, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if s.closed {
		return nil, ErrClosed
	}
	e, ok := s.index[string(key)]
	if !ok {
		return nil, ErrNotFound
	}
	buf := make([]byte, e.size)
	if _, err := s.files[e.seg].ReadAt(buf, e.off); err != nil {
		return nil, err
	}
	rec, _, err := readRecord(bytes.NewReader(buf))
	if err != nil {
		return nil, err
	}
	return rec.value, nil
}

// Len returns the number of live keys.
func (s *Store) Len() int {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return len(s.index)
}

// Sync flushes the active segment to stable storage.
func (s *Store) Sync() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return ErrClosed
	}
	return s.active.Sync()
}

// Close syncs and closes all files.
func (s *Store) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return nil
	}
	s.closed = true
	err := s.active.Sync()
	for _, f := range s.files {
		if cerr := f.Close(); err == nil {
			err = cerr
		}
	}
	return err
}
