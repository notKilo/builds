package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestSchedulerRecordsAndNotifies(t *testing.T) {
	target := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(503) }))
	defer target.Close()
	hooks := make(chan struct{}, 4)
	hook := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { hooks <- struct{}{} }))
	defer hook.Close()

	st := newTestStore(t)
	s := &Scheduler{
		Checks:    []Check{{Name: "svc", Type: "http", Target: target.URL, Interval: Duration(time.Second), Timeout: Duration(time.Second)}},
		Store:     st,
		Notifier:  &Notifier{URL: hook.URL},
		FlapAfter: 1,
	}
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan struct{})
	go func() { s.Run(ctx); close(done) }()

	select {
	case <-hooks:
	case <-time.After(3 * time.Second):
		t.Fatal("no webhook received")
	}
	cancel()
	select {
	case <-done:
	case <-time.After(3 * time.Second):
		t.Fatal("scheduler did not stop")
	}
	rs, _ := st.Recent("svc", 5)
	if len(rs) == 0 || rs[0].Up {
		t.Errorf("results %+v", rs)
	}
	if r, ok := s.Last("svc"); !ok || r.Up {
		t.Errorf("last %+v", r)
	}
}
