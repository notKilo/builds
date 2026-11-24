package report

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
	"time"

	"github.com/notkilo/builds/projects/hammer/internal/runner"
	"github.com/notkilo/builds/projects/hammer/internal/stats"
)

func sample() *runner.Result {
	rec := &stats.Recorder{}
	for _, m := range []int{1, 2, 3, 4, 50} {
		rec.Add(time.Duration(m) * time.Millisecond)
	}
	return &runner.Result{
		Elapsed: time.Second, Latency: rec, Total: 6,
		Statuses: map[int]int{200: 4, 500: 1},
		Errors:   map[string]int{"boom": 1},
	}
}

func TestText(t *testing.T) {
	var b bytes.Buffer
	Text(&b, sample(), 5)
	for _, want := range []string{"Requests:   6", "200: 4", "500: 1", "1x boom", "p99", "#"} {
		if !strings.Contains(b.String(), want) {
			t.Errorf("missing %q in:\n%s", want, b.String())
		}
	}
}

func TestJSON(t *testing.T) {
	var b bytes.Buffer
	if err := JSON(&b, sample(), 5); err != nil {
		t.Fatal(err)
	}
	var got struct {
		Total   int                `json:"total"`
		Latency map[string]float64 `json:"latency_ms"`
	}
	if err := json.Unmarshal(b.Bytes(), &got); err != nil {
		t.Fatalf("invalid json: %v\n%s", err, b.String())
	}
	if got.Total != 6 || got.Latency["max"] != 50 {
		t.Fatalf("unexpected report: %+v", got)
	}
}
