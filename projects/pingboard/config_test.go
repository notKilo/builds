package main

import (
	"strings"
	"testing"
	"time"
)

func TestParseConfigDefaults(t *testing.T) {
	c, err := ParseConfig([]byte(`{"checks":[{"name":"a","type":"http","target":"http://x"}]}`))
	if err != nil {
		t.Fatal(err)
	}
	if c.Listen != ":8080" || c.FlapAfter != 2 {
		t.Errorf("defaults not applied: %+v", c)
	}
	if c.Checks[0].Interval.D() != 30*time.Second || c.Checks[0].Timeout.D() != 10*time.Second {
		t.Errorf("check defaults wrong: %+v", c.Checks[0])
	}
}

func TestParseConfigErrors(t *testing.T) {
	cases := map[string]string{
		`{"checks":[{"type":"http","target":"x"}]}`:                                                  "name is required",
		`{"checks":[{"name":"a","type":"ftp","target":"x"}]}`:                                        "http or tcp",
		`{"checks":[{"name":"a","type":"tcp"}]}`:                                                     "target is required",
		`{"checks":[{"name":"a","type":"tcp","target":"x"},{"name":"a","type":"tcp","target":"y"}]}`: "duplicate",
		`{"checks":[{"name":"a","type":"tcp","target":"x","interval":"10ms"}]}`:                      ">= 1s",
		`{"bogus":1}`: "unknown field",
		`{"checks":[{"name":"a","type":"tcp","target":"x","interval":5}]}`: "duration",
	}
	for in, want := range cases {
		_, err := ParseConfig([]byte(in))
		if err == nil || !strings.Contains(err.Error(), want) {
			t.Errorf("%s: got %v, want error containing %q", in, err, want)
		}
	}
}
