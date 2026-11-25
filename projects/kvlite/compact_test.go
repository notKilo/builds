package kvlite

import (
	"fmt"
	"os"
	"testing"
)

func TestSegmentRotation(t *testing.T) {
	dir := t.TempDir()
	s := open(t, dir, Options{MaxSegmentSize: 200})
	for i := 0; i < 50; i++ {
		s.Put([]byte(fmt.Sprintf("key%02d", i)), []byte("some-value-bytes"))
	}
	st, _ := s.Stats()
	if st.Segments < 5 {
		t.Fatalf("expected rotation, got %d segments", st.Segments)
	}
	s.Close()
	s = open(t, dir, Options{MaxSegmentSize: 200})
	defer s.Close()
	if s.Len() != 50 {
		t.Fatalf("Len = %d after reopen", s.Len())
	}
	mustGet(t, s, "key49", "some-value-bytes")
}

func TestCompactDropsDeadData(t *testing.T) {
	dir := t.TempDir()
	s := open(t, dir, Options{MaxSegmentSize: 300})
	for round := 0; round < 10; round++ {
		for i := 0; i < 20; i++ {
			s.Put([]byte(fmt.Sprintf("k%02d", i)), []byte(fmt.Sprintf("round-%d", round)))
		}
	}
	for i := 0; i < 10; i++ {
		s.Delete([]byte(fmt.Sprintf("k%02d", i)))
	}
	before, _ := s.Stats()
	if err := s.Compact(); err != nil {
		t.Fatal(err)
	}
	after, _ := s.Stats()
	if after.DiskBytes >= before.DiskBytes/2 {
		t.Errorf("disk %d -> %d, expected a big reduction", before.DiskBytes, after.DiskBytes)
	}
	if after.Keys != 10 {
		t.Errorf("keys = %d, want 10", after.Keys)
	}
	check := func(s *Store) {
		for i := 10; i < 20; i++ {
			mustGet(t, s, fmt.Sprintf("k%02d", i), "round-9")
		}
		if _, err := s.Get([]byte("k03")); err == nil {
			t.Error("deleted key resurrected")
		}
	}
	check(s)
	// Writes after compaction land in a newer segment and survive reopen.
	s.Put([]byte("k10"), []byte("fresh"))
	s.Close()
	s = open(t, dir, Options{MaxSegmentSize: 300})
	defer s.Close()
	mustGet(t, s, "k10", "fresh")
	mustGet(t, s, "k19", "round-9")
	if _, err := os.Stat(dir + "/compact.tmp"); err == nil {
		t.Error("temp file left behind")
	}
}

func TestCompactNoopOnCleanStore(t *testing.T) {
	s := open(t, t.TempDir(), Options{})
	defer s.Close()
	s.Put([]byte("a"), []byte("1"))
	if err := s.Compact(); err != nil {
		t.Fatal(err)
	}
	mustGet(t, s, "a", "1")
}

func TestCompactTwice(t *testing.T) {
	s := open(t, t.TempDir(), Options{MaxSegmentSize: 100})
	defer s.Close()
	for i := 0; i < 40; i++ {
		s.Put([]byte("same"), []byte(fmt.Sprintf("v%d", i)))
	}
	for i := 0; i < 2; i++ {
		if err := s.Compact(); err != nil {
			t.Fatal(err)
		}
		mustGet(t, s, "same", "v39")
	}
}
