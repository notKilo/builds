package main

import (
	"testing"
	"time"
)

func newTestStore(t *testing.T) *Store {
	t.Helper()
	s, err := OpenStore(":memory:")
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { s.Close() })
	return s
}

func res(name string, at time.Time, up bool, ms int) Result {
	return Result{Check: name, Time: at, Up: up, Latency: time.Duration(ms) * time.Millisecond}
}

func TestInsertAndRecent(t *testing.T) {
	s := newTestStore(t)
	base := time.Now().Truncate(time.Millisecond)
	for i := 0; i < 5; i++ {
		if err := s.Insert(res("a", base.Add(time.Duration(i)*time.Second), i%2 == 0, 10+i)); err != nil {
			t.Fatal(err)
		}
	}
	s.Insert(res("b", base, true, 1))
	got, err := s.Recent("a", 3)
	if err != nil {
		t.Fatal(err)
	}
	if len(got) != 3 || !got[0].Time.Equal(base.Add(4*time.Second)) || got[0].Latency != 14*time.Millisecond {
		t.Errorf("unexpected recent: %+v", got)
	}
}

func TestStats(t *testing.T) {
	s := newTestStore(t)
	now := time.Now()
	for i := 1; i <= 10; i++ {
		s.Insert(res("a", now, true, i*10))
	}
	s.Insert(res("a", now, false, 5000))
	s.Insert(res("a", now.Add(-48*time.Hour), true, 1)) // outside window
	st, err := s.Stats("a", now.Add(-time.Hour))
	if err != nil {
		t.Fatal(err)
	}
	if st.Total != 11 || st.Up != 10 {
		t.Errorf("counts: %+v", st)
	}
	if st.UptimePct < 90.9 || st.UptimePct > 91 {
		t.Errorf("uptime = %v", st.UptimePct)
	}
	if st.P50 != 50*time.Millisecond || st.P95 != 100*time.Millisecond {
		t.Errorf("percentiles p50=%v p95=%v", st.P50, st.P95)
	}
}

func TestStatsEmpty(t *testing.T) {
	st, err := newTestStore(t).Stats("none", time.Now().Add(-time.Hour))
	if err != nil || st.Total != 0 || st.P95 != 0 {
		t.Errorf("got %+v, %v", st, err)
	}
}

func TestPrune(t *testing.T) {
	s := newTestStore(t)
	now := time.Now()
	s.Insert(res("a", now.Add(-2*time.Hour), true, 1))
	s.Insert(res("a", now, true, 1))
	n, err := s.Prune(now.Add(-time.Hour))
	if err != nil || n != 1 {
		t.Fatalf("pruned %d, %v", n, err)
	}
	if got, _ := s.Recent("a", 10); len(got) != 1 {
		t.Errorf("remaining %d", len(got))
	}
}
