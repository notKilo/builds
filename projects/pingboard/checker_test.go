package main

import (
	"context"
	"net"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func httpCheck(url string) Check {
	return Check{Name: "t", Type: "http", Target: url, Timeout: Duration(2 * time.Second)}
}

func TestHTTPUp(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { w.Write([]byte("hello world")) }))
	defer s.Close()
	c := httpCheck(s.URL)
	c.ExpectBody = "world"
	r := Run(context.Background(), c)
	if !r.Up || r.Status != 200 {
		t.Fatalf("want up, got %+v", r)
	}
}

func TestHTTPFailures(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/err" {
			w.WriteHeader(500)
		}
		w.Write([]byte("abc"))
	}))
	defer s.Close()

	if r := Run(context.Background(), httpCheck(s.URL+"/err")); r.Up || r.Status != 500 {
		t.Errorf("500 should be down: %+v", r)
	}
	c := httpCheck(s.URL + "/err")
	c.ExpectStatus = 500
	if r := Run(context.Background(), c); !r.Up {
		t.Errorf("expected 500 should be up: %+v", r)
	}
	c = httpCheck(s.URL)
	c.ExpectBody = "zzz"
	if r := Run(context.Background(), c); r.Up {
		t.Errorf("missing body should be down: %+v", r)
	}
}

func TestHTTPTimeout(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) { time.Sleep(300 * time.Millisecond) }))
	defer s.Close()
	c := httpCheck(s.URL)
	c.Timeout = Duration(50 * time.Millisecond)
	if r := Run(context.Background(), c); r.Up || r.Error == "" {
		t.Errorf("want timeout failure: %+v", r)
	}
}

func TestTCP(t *testing.T) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal(err)
	}
	addr := l.Addr().String()
	c := Check{Name: "t", Type: "tcp", Target: addr, Timeout: Duration(time.Second)}
	if r := Run(context.Background(), c); !r.Up {
		t.Errorf("listening port should be up: %+v", r)
	}
	l.Close()
	if r := Run(context.Background(), c); r.Up {
		t.Errorf("closed port should be down: %+v", r)
	}
}
