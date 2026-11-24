package stats

import (
	"strings"
	"testing"
	"time"
)

func rec(ms ...int) *Recorder {
	r := &Recorder{}
	for _, m := range ms {
		r.Add(time.Duration(m) * time.Millisecond)
	}
	return r
}

func TestSummarize(t *testing.T) {
	r := &Recorder{}
	for i := 1; i <= 100; i++ {
		r.Add(time.Duration(i) * time.Millisecond)
	}
	s := r.Summarize()
	if s.Count != 100 || s.Min != time.Millisecond || s.Max != 100*time.Millisecond {
		t.Fatalf("bad bounds: %+v", s)
	}
	if s.P50 != 50*time.Millisecond || s.P90 != 90*time.Millisecond || s.P99 != 99*time.Millisecond {
		t.Fatalf("bad percentiles: %+v", s)
	}
}

func TestSummarizeEmpty(t *testing.T) {
	if s := (&Recorder{}).Summarize(); s.Count != 0 {
		t.Fatalf("expected zero summary, got %+v", s)
	}
}

func TestMerge(t *testing.T) {
	a, b := rec(1, 2), rec(3)
	a.Merge(b)
	if a.Count() != 3 {
		t.Fatalf("count = %d", a.Count())
	}
}

func TestHistogramCountsAll(t *testing.T) {
	r := rec(1, 2, 3, 10, 20, 50, 50, 100)
	total := 0
	for _, b := range r.Histogram(5) {
		total += b.Count
	}
	if total != r.Count() {
		t.Fatalf("histogram lost samples: %d != %d", total, r.Count())
	}
}

func TestHistogramSingleValue(t *testing.T) {
	b := rec(5, 5, 5).Histogram(4)
	if len(b) != 1 || b[0].Count != 3 {
		t.Fatalf("unexpected: %+v", b)
	}
}

func TestRenderHistogram(t *testing.T) {
	out := RenderHistogram(rec(1, 1, 1, 100).Histogram(2), 10)
	if !strings.Contains(out, "##########") {
		t.Fatalf("expected full bar:\n%s", out)
	}
}
