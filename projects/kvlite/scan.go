package kvlite

import (
	"bytes"
	"sort"
)

// Keys returns the live keys that start with prefix, in sorted order.
// An empty prefix returns every key.
func (s *Store) Keys(prefix []byte) ([][]byte, error) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	if s.closed {
		return nil, ErrClosed
	}
	var keys [][]byte
	for k := range s.index {
		if bytes.HasPrefix([]byte(k), prefix) {
			keys = append(keys, []byte(k))
		}
	}
	sort.Slice(keys, func(i, j int) bool { return bytes.Compare(keys[i], keys[j]) < 0 })
	return keys, nil
}

// Scan calls fn with each key/value pair whose key starts with prefix, in key
// order. Iteration stops early if fn returns false. fn must not call back
// into the store's write methods.
func (s *Store) Scan(prefix []byte, fn func(key, value []byte) bool) error {
	keys, err := s.Keys(prefix)
	if err != nil {
		return err
	}
	for _, k := range keys {
		v, err := s.Get(k)
		if err == ErrNotFound { // deleted since Keys ran
			continue
		}
		if err != nil {
			return err
		}
		if !fn(k, v) {
			return nil
		}
	}
	return nil
}
