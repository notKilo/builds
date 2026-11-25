package gitstats

import (
	"encoding/json"
	"fmt"
	"io"
	"strings"
	"text/tabwriter"
	"time"
)

// Options controls report generation.
type Options struct {
	Top      int           // rows per section (0 = all)
	Depth    int           // directory depth for bus factor
	HalfLife time.Duration // hotspot recency half-life
	Now      time.Time
}

// Report is the full analysis of a history.
type Report struct {
	Commits   int          `json:"commits"`
	First     time.Time    `json:"first_commit"`
	Last      time.Time    `json:"last_commit"`
	Churn     []FileStat   `json:"top_churn"`
	Hotspots  []FileStat   `json:"hotspots"`
	BusFactor []BusFactor  `json:"bus_factor"`
	Authors   []AuthorStat `json:"authors"`
	Activity  Activity     `json:"activity"`
}

// Build analyses commits (newest first) into a Report.
func Build(commits []Commit, o Options) Report {
	if o.Now.IsZero() {
		o.Now = time.Now()
	}
	if o.Depth < 1 {
		o.Depth = 1
	}
	if o.HalfLife <= 0 {
		o.HalfLife = 90 * 24 * time.Hour
	}
	r := Report{Commits: len(commits)}
	for i, c := range commits {
		if i == 0 || c.Time.Before(r.First) {
			r.First = c.Time
		}
		if i == 0 || c.Time.After(r.Last) {
			r.Last = c.Time
		}
	}
	files := Files(commits, o.Now, o.HalfLife)
	r.Churn = TopByChurn(files, o.Top)
	r.Hotspots = TopByHotspot(files, o.Top)
	r.BusFactor = BusFactors(commits, o.Depth)
	if o.Top > 0 && len(r.BusFactor) > o.Top {
		r.BusFactor = r.BusFactor[:o.Top]
	}
	r.Authors = Authors(commits)
	if o.Top > 0 && len(r.Authors) > o.Top {
		r.Authors = r.Authors[:o.Top]
	}
	r.Activity = ActivityOf(commits)
	return r
}

// WriteJSON writes the report as indented JSON.
func (r Report) WriteJSON(w io.Writer) error {
	enc := json.NewEncoder(w)
	enc.SetIndent("", "  ")
	return enc.Encode(r)
}

// WriteText writes aligned plain-text tables.
func (r Report) WriteText(w io.Writer) error {
	if r.Commits == 0 {
		_, err := fmt.Fprintln(w, "no commits found")
		return err
	}
	fmt.Fprintf(w, "%d commits, %s to %s\n", r.Commits, r.First.Format("2006-01-02"), r.Last.Format("2006-01-02"))

	section := func(title string, header string, rows []string) {
		fmt.Fprintf(w, "\n%s\n", title)
		tw := tabwriter.NewWriter(w, 0, 4, 2, ' ', 0)
		fmt.Fprintln(tw, header)
		for _, row := range rows {
			fmt.Fprintln(tw, row)
		}
		tw.Flush()
	}
	var rows []string
	for _, f := range r.Churn {
		rows = append(rows, fmt.Sprintf("%s\t%d\t%d\t+%d/-%d\t%d", f.Path, f.Churn(), f.Commits, f.Added, f.Deleted, f.Authors))
	}
	section("Top churn", "FILE\tLINES\tCOMMITS\tDIFF\tAUTHORS", rows)

	rows = nil
	for _, f := range r.Hotspots {
		rows = append(rows, fmt.Sprintf("%s\t%.2f\t%d\t%s", f.Path, f.Score, f.Commits, f.Last.Format("2006-01-02")))
	}
	section("Hotspots (recency-weighted)", "FILE\tSCORE\tCOMMITS\tLAST CHANGE", rows)

	rows = nil
	for _, b := range r.BusFactor {
		rows = append(rows, fmt.Sprintf("%s\t%d\t%d\t%s", b.Dir, b.Factor, b.Lines, strings.Join(b.Owners, ", ")))
	}
	section("Bus factor by directory", "DIR\tFACTOR\tLINES\tOWNERS", rows)

	rows = nil
	for _, a := range r.Authors {
		rows = append(rows, fmt.Sprintf("%s <%s>\t%d\t+%d/-%d\t%s", a.Name, a.Email, a.Commits, a.Added, a.Deleted, a.Last.Format("2006-01-02")))
	}
	section("Authors", "AUTHOR\tCOMMITS\tDIFF\tLAST", rows)

	days := []string{"Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"}
	rows = nil
	maxN := 1
	for _, n := range r.Activity.ByWeekday {
		maxN = max(maxN, n)
	}
	for i, n := range r.Activity.ByWeekday {
		rows = append(rows, fmt.Sprintf("%s\t%d\t%s", days[i], n, strings.Repeat("#", n*30/maxN)))
	}
	section("Commits by weekday", "DAY\tCOMMITS\t", rows)
	return nil
}
