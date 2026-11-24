// Package stats collects latency samples and summarizes them.
package stats

import (
	"fmt"
	"sort"
	"strings"
	"time"
)

// Recorder accumulates latency samples. It is not safe for concurrent use;
// callers merge per-worker recorders with Merge.
type Recorder struct {
	samples []time.Duration
}

// Add records one latency sample.
func (r *Recorder) Add(d time.Duration) { r.samples = append(r.samples, d) }

// Merge appends all samples from other.
func (r *Recorder) Merge(other *Recorder) { r.samples = append(r.samples, other.samples...) }

// Count returns the number of samples.
func (r *Recorder) Count() int { return len(r.samples) }

// Summary holds aggregate latency figures.
type Summary struct {
	Count              int
	Min, Max, Mean     time.Duration
	P50, P90, P95, P99 time.Duration
}

// Summarize computes min/max/mean and percentiles (nearest-rank).
func (r *Recorder) Summarize() Summary {
	if len(r.samples) == 0 {
		return Summary{}
	}
	s := append([]time.Duration(nil), r.samples...)
	sort.Slice(s, func(i, j int) bool { return s[i] < s[j] })
	var total time.Duration
	for _, d := range s {
		total += d
	}
	return Summary{
		Count: len(s),
		Min:   s[0],
		Max:   s[len(s)-1],
		Mean:  total / time.Duration(len(s)),
		P50:   percentile(s, 50),
		P90:   percentile(s, 90),
		P95:   percentile(s, 95),
		P99:   percentile(s, 99),
	}
}

// percentile uses the nearest-rank method on sorted data.
func percentile(sorted []time.Duration, p float64) time.Duration {
	rank := int(p/100*float64(len(sorted)) + 0.999999999)
	if rank < 1 {
		rank = 1
	}
	if rank > len(sorted) {
		rank = len(sorted)
	}
	return sorted[rank-1]
}

// Bucket is one histogram bin covering [Lo, Hi).
type Bucket struct {
	Lo, Hi time.Duration
	Count  int
}

// Histogram splits the sample range into n equal-width buckets.
func (r *Recorder) Histogram(n int) []Bucket {
	if len(r.samples) == 0 || n < 1 {
		return nil
	}
	sum := r.Summarize()
	span := sum.Max - sum.Min
	if span == 0 {
		return []Bucket{{Lo: sum.Min, Hi: sum.Max, Count: len(r.samples)}}
	}
	width := span/time.Duration(n) + 1
	buckets := make([]Bucket, n)
	for i := range buckets {
		buckets[i].Lo = sum.Min + time.Duration(i)*width
		buckets[i].Hi = buckets[i].Lo + width
	}
	for _, d := range r.samples {
		i := int((d - sum.Min) / width)
		if i >= n {
			i = n - 1
		}
		buckets[i].Count++
	}
	return buckets
}

// RenderHistogram draws buckets as horizontal bars at most barWidth wide.
func RenderHistogram(buckets []Bucket, barWidth int) string {
	max := 0
	for _, b := range buckets {
		if b.Count > max {
			max = b.Count
		}
	}
	var sb strings.Builder
	for _, b := range buckets {
		n := 0
		if max > 0 {
			n = b.Count * barWidth / max
		}
		if b.Count > 0 && n == 0 {
			n = 1
		}
		fmt.Fprintf(&sb, "%10s %s %d\n", b.Lo.Round(time.Microsecond), strings.Repeat("#", n), b.Count)
	}
	return sb.String()
}
