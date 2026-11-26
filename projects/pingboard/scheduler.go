package main

import (
	"context"
	"log"
	"math/rand"
	"sync"
	"time"
)

// Scheduler runs each check on its own interval and records results.
type Scheduler struct {
	Checks    []Check
	Store     *Store
	Notifier  *Notifier
	FlapAfter int

	mu      sync.Mutex
	tracker *Tracker
	last    map[string]Result
}

// Last returns the most recent in-memory result for a check.
func (s *Scheduler) Last(name string) (Result, bool) {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.last[name]
	return r, ok
}

// Run blocks until ctx is cancelled, then waits for in-flight checks.
func (s *Scheduler) Run(ctx context.Context) {
	s.tracker = NewTracker(s.FlapAfter)
	s.last = map[string]Result{}
	var wg sync.WaitGroup
	for _, c := range s.Checks {
		wg.Add(1)
		go func(c Check) {
			defer wg.Done()
			s.loop(ctx, c)
		}(c)
	}
	wg.Wait()
}

func (s *Scheduler) loop(ctx context.Context, c Check) {
	// jitter the first run so checks do not all fire at once
	first := time.Duration(rand.Int63n(int64(c.Interval.D()/4) + 1))
	timer := time.NewTimer(first)
	defer timer.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-timer.C:
		}
		s.runOnce(ctx, c)
		timer.Reset(c.Interval.D())
	}
}

func (s *Scheduler) runOnce(ctx context.Context, c Check) {
	r := Run(ctx, c)
	if ctx.Err() != nil {
		return // shutting down: do not record a cancelled check as down
	}
	if err := s.Store.Insert(r); err != nil {
		log.Printf("store %s: %v", c.Name, err)
	}
	s.mu.Lock()
	s.last[c.Name] = r
	tr := s.tracker.Observe(r)
	s.mu.Unlock()
	if tr != nil {
		log.Printf("%s -> up=%v %s", tr.Check, tr.Up, tr.Error)
		if err := s.Notifier.Notify(ctx, *tr); err != nil {
			log.Printf("notify %s: %v", c.Name, err)
		}
	}
}
