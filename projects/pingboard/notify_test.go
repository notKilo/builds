package main

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

func TestNotify(t *testing.T) {
	var got webhookPayload
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		json.NewDecoder(r.Body).Decode(&got)
	}))
	defer s.Close()
	n := &Notifier{URL: s.URL}
	if err := n.Notify(context.Background(), Transition{Check: "api", Up: false, Error: "timeout"}); err != nil {
		t.Fatal(err)
	}
	if got.State != "down" || !strings.Contains(got.Text, "api is DOWN: timeout") {
		t.Errorf("payload %+v", got)
	}
}

func TestNotifyErrorsAndNoop(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.WriteHeader(500) }))
	defer s.Close()
	if err := (&Notifier{URL: s.URL}).Notify(context.Background(), Transition{Check: "a", Up: true}); err == nil {
		t.Error("want error on 500")
	}
	if err := (*Notifier)(nil).Notify(context.Background(), Transition{}); err != nil {
		t.Error("nil notifier should be a no-op")
	}
}
