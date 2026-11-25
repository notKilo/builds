package kvlite

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"testing"
)

func open(t *testing.T, dir string, opts Options) *Store {
	t.Helper()
	s, err := Open(dir, opts)
	if err != nil {
		t.Fatal(err)
	}
	return s
}

func mustGet(t *testing.T, s *Store, k, want string) {
	t.Helper()
	got, err := s.Get([]byte(k))
	if err != nil || string(got) != want {
		t.Fatalf("Get(%q) = %q, %v; want %q", k, got, err, want)
	}
}

func TestPutGetDelete(t *testing.T) {
	s := open(t, t.TempDir(), Options{})
	defer s.Close()
	if _, err := s.Get([]byte("x")); !errors.Is(err, ErrNotFound) {
		t.Fatalf("missing key err = %v", err)
	}
	s.Put([]byte("x"), []byte("1"))
	s.Put([]byte("x"), []byte("2"))
	mustGet(t, s, "x", "2")
	if err := s.Delete([]byte("x")); err != nil {
		t.Fatal(err)
	}
	if _, err := s.Get([]byte("x")); !errors.Is(err, ErrNotFound) {
		t.Fatalf("after delete err = %v", err)
	}
	if err := s.Delete([]byte("never-existed")); err != nil {
		t.Fatal(err)
	}
}

func TestValidation(t *testing.T) {
	s := open(t, t.TempDir(), Options{})
	defer s.Close()
	if err := s.Put(nil, []byte("v")); !errors.Is(err, ErrEmptyKey) {
		t.Errorf("empty key err = %v", err)
	}
	if err := s.Put(make([]byte, MaxKeySize+1), nil); !errors.Is(err, ErrKeyTooBig) {
		t.Errorf("big key err = %v", err)
	}
}

func TestReopenRecoversState(t *testing.T) {
	dir := t.TempDir()
	s := open(t, dir, Options{})
	for i := 0; i < 100; i++ {
		s.Put([]byte(fmt.Sprintf("k%d", i)), []byte(fmt.Sprintf("v%d", i)))
	}
	s.Put([]byte("k5"), []byte("changed"))
	s.Delete([]byte("k7"))
	s.Close()

	s = open(t, dir, Options{})
	defer s.Close()
	if s.Len() != 99 {
		t.Errorf("Len = %d, want 99", s.Len())
	}
	mustGet(t, s, "k5", "changed")
	mustGet(t, s, "k99", "v99")
	if _, err := s.Get([]byte("k7")); !errors.Is(err, ErrNotFound) {
		t.Errorf("k7 should stay deleted, err = %v", err)
	}
}

func TestClosedStore(t *testing.T) {
	s := open(t, t.TempDir(), Options{})
	s.Close()
	if err := s.Put([]byte("a"), nil); !errors.Is(err, ErrClosed) {
		t.Errorf("Put after close = %v", err)
	}
	if _, err := s.Get([]byte("a")); !errors.Is(err, ErrClosed) {
		t.Errorf("Get after close = %v", err)
	}
}

func TestTornTailIsTruncated(t *testing.T) {
	dir := t.TempDir()
	s := open(t, dir, Options{})
	s.Put([]byte("a"), []byte("1"))
	s.Put([]byte("b"), []byte("2"))
	s.Close()

	path := filepath.Join(dir, segmentName(1))
	info, _ := os.Stat(path)
	// Chop into the middle of the last record.
	if err := os.Truncate(path, info.Size()-3); err != nil {
		t.Fatal(err)
	}
	s = open(t, dir, Options{})
	mustGet(t, s, "a", "1")
	if _, err := s.Get([]byte("b")); !errors.Is(err, ErrNotFound) {
		t.Fatalf("torn record must be dropped, err = %v", err)
	}
	// The store must keep working and persist new writes after the repair.
	s.Put([]byte("c"), []byte("3"))
	s.Close()
	s = open(t, dir, Options{})
	defer s.Close()
	mustGet(t, s, "a", "1")
	mustGet(t, s, "c", "3")
}

func TestCorruptTailBytesAreTruncated(t *testing.T) {
	dir := t.TempDir()
	s := open(t, dir, Options{})
	s.Put([]byte("a"), []byte("1"))
	s.Close()
	path := filepath.Join(dir, segmentName(1))
	f, _ := os.OpenFile(path, os.O_APPEND|os.O_WRONLY, 0)
	f.Write([]byte("garbage-not-a-record"))
	f.Close()

	s = open(t, dir, Options{})
	defer s.Close()
	mustGet(t, s, "a", "1")
	if s.Len() != 1 {
		t.Errorf("Len = %d", s.Len())
	}
}
