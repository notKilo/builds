package kvlite

import (
	"io"
	"os"
	"path/filepath"
	"sort"
)

// Stats describes the on-disk footprint of a store.
type Stats struct {
	Keys      int
	Segments  int
	DiskBytes int64 // total size of all segments
	LiveBytes int64 // bytes occupied by the latest record of each key
}

// Garbage returns the fraction of disk space taken by dead records.
func (st Stats) Garbage() float64 {
	if st.DiskBytes == 0 {
		return 0
	}
	return 1 - float64(st.LiveBytes)/float64(st.DiskBytes)
}

func (s *Store) Stats() (Stats, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if s.closed {
		return Stats{}, ErrClosed
	}
	st := Stats{Keys: len(s.index), Segments: len(s.files)}
	for _, e := range s.index {
		st.LiveBytes += int64(e.size)
	}
	for _, f := range s.files {
		info, err := f.Stat()
		if err != nil {
			return Stats{}, err
		}
		st.DiskBytes += info.Size()
	}
	return st, nil
}

// Compact rewrites all closed segments into one that holds only the latest
// record of each live key, then removes the originals. Reads and writes are
// blocked while it runs.
//
// Crash safety: the merged file is built under a temporary name and renamed
// over the oldest segment, then the remaining old segments are deleted oldest
// first. Replaying any intermediate state gives the same key/value set,
// because the merged segment sorts before every leftover original.
func (s *Store) Compact() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.closed {
		return ErrClosed
	}
	if s.actLen > 0 {
		if err := s.roll(); err != nil {
			return err
		}
	}
	var old []uint32
	for id := range s.files {
		if id < s.actID {
			old = append(old, id)
		}
	}
	if len(old) < 2 && !s.hasGarbage(old) {
		return nil
	}
	sort.Slice(old, func(i, j int) bool { return old[i] < old[j] })
	target := old[0]

	// Visit live entries in disk order so the merged file keeps write order.
	type kv struct {
		key string
		e   entry
	}
	var live []kv
	for k, e := range s.index {
		if e.seg < s.actID {
			live = append(live, kv{k, e})
		}
	}
	sort.Slice(live, func(i, j int) bool {
		a, b := live[i].e, live[j].e
		if a.seg != b.seg {
			return a.seg < b.seg
		}
		return a.off < b.off
	})

	tmpPath := filepath.Join(s.dir, "compact.tmp")
	tmp, err := os.OpenFile(tmpPath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0o644)
	if err != nil {
		return err
	}
	fail := func(err error) error {
		tmp.Close()
		os.Remove(tmpPath)
		return err
	}
	newEntries := make(map[string]entry, len(live))
	var off int64
	for _, item := range live {
		src := io.NewSectionReader(s.files[item.e.seg], item.e.off, int64(item.e.size))
		if _, err := io.Copy(tmp, src); err != nil {
			return fail(err)
		}
		newEntries[item.key] = entry{seg: target, off: off, size: item.e.size}
		off += int64(item.e.size)
	}
	if err := tmp.Sync(); err != nil {
		return fail(err)
	}
	// Swap the merged file in, then drop the other originals oldest first.
	s.files[target].Close()
	if err := os.Rename(tmpPath, filepath.Join(s.dir, segmentName(target))); err != nil {
		tmp.Close()
		os.Remove(tmpPath)
		// Restore a usable handle for the segment we just closed.
		f, oerr := os.OpenFile(filepath.Join(s.dir, segmentName(target)), os.O_RDWR, 0o644)
		if oerr != nil {
			return oerr
		}
		s.files[target] = f
		return err
	}
	s.files[target] = tmp
	for _, id := range old[1:] {
		s.files[id].Close()
		delete(s.files, id)
		if err := os.Remove(filepath.Join(s.dir, segmentName(id))); err != nil {
			return err
		}
	}
	for k, e := range newEntries {
		s.index[k] = e
	}
	return nil
}

// hasGarbage reports whether the given closed segments contain any record
// that is no longer the latest for its key. The caller holds s.mu.
func (s *Store) hasGarbage(ids []uint32) bool {
	var live int64
	for _, e := range s.index {
		for _, id := range ids {
			if e.seg == id {
				live += int64(e.size)
			}
		}
	}
	var disk int64
	for _, id := range ids {
		if info, err := s.files[id].Stat(); err == nil {
			disk += info.Size()
		}
	}
	return disk > live
}
