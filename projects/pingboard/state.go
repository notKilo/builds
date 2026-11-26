package main

// Transition reports a confirmed state change for a check.
type Transition struct {
	Check string
	Up    bool
	Error string
}

// Tracker debounces results: the state flips only after FlapAfter
// consecutive results disagree with the current state.
type Tracker struct {
	flapAfter int
	state     map[string]*checkState
}

type checkState struct {
	known   bool
	up      bool
	pending int
}

func NewTracker(flapAfter int) *Tracker {
	if flapAfter < 1 {
		flapAfter = 1
	}
	return &Tracker{flapAfter: flapAfter, state: map[string]*checkState{}}
}

// Observe feeds a result and returns a transition if the state changed.
// The first observation establishes state; it only reports a transition
// if the check starts out down (so a fresh start with a dead target alerts).
func (t *Tracker) Observe(r Result) *Transition {
	s := t.state[r.Check]
	if s == nil {
		s = &checkState{known: true, up: r.Up}
		t.state[r.Check] = s
		if !r.Up {
			return &Transition{Check: r.Check, Up: false, Error: r.Error}
		}
		return nil
	}
	if r.Up == s.up {
		s.pending = 0
		return nil
	}
	s.pending++
	if s.pending < t.flapAfter {
		return nil
	}
	s.up, s.pending = r.Up, 0
	return &Transition{Check: r.Check, Up: r.Up, Error: r.Error}
}
