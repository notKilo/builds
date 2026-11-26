package main

import "testing"

func obs(t *testing.T, tr *Tracker, ups ...bool) []*Transition {
	var out []*Transition
	for _, u := range ups {
		out = append(out, tr.Observe(Result{Check: "a", Up: u}))
	}
	return out
}

func TestTrackerDebounce(t *testing.T) {
	tr := NewTracker(2)
	got := obs(t, tr, true, false, true, false, false, true, true)
	want := []bool{false, false, false, false, true, false, true} // true = transition expected
	for i, w := range want {
		if (got[i] != nil) != w {
			t.Errorf("step %d: transition=%v, want %v", i, got[i] != nil, w)
		}
	}
	if got[4].Up || !got[6].Up {
		t.Errorf("wrong directions: %+v %+v", got[4], got[6])
	}
}

func TestTrackerStartsDown(t *testing.T) {
	got := NewTracker(3).Observe(Result{Check: "a", Up: false, Error: "boom"})
	if got == nil || got.Up || got.Error != "boom" {
		t.Errorf("got %+v", got)
	}
}
