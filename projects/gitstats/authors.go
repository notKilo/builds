package gitstats

import (
	"sort"
	"time"
)

// AuthorStat summarises one author's contribution.
type AuthorStat struct {
	Name    string    `json:"name"`
	Email   string    `json:"email"`
	Commits int       `json:"commits"`
	Added   int       `json:"added"`
	Deleted int       `json:"deleted"`
	First   time.Time `json:"first_commit"`
	Last    time.Time `json:"last_commit"`
}

// Authors aggregates by email, sorted by commit count then email.
func Authors(commits []Commit) []AuthorStat {
	m := map[string]*AuthorStat{}
	for _, c := range commits {
		a := m[c.Email]
		if a == nil {
			a = &AuthorStat{Name: c.Author, Email: c.Email, First: c.Time, Last: c.Time}
			m[c.Email] = a
		}
		a.Commits++
		for _, f := range c.Files {
			a.Added += f.Added
			a.Deleted += f.Deleted
		}
		if c.Time.Before(a.First) {
			a.First = c.Time
		}
		if c.Time.After(a.Last) {
			a.Last = c.Time
			a.Name = c.Author // prefer the most recent display name
		}
	}
	out := make([]AuthorStat, 0, len(m))
	for _, a := range m {
		out = append(out, *a)
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Commits != out[j].Commits {
			return out[i].Commits > out[j].Commits
		}
		return out[i].Email < out[j].Email
	})
	return out
}

// BusFactor describes knowledge concentration in a directory.
type BusFactor struct {
	Dir    string   `json:"dir"`
	Factor int      `json:"bus_factor"` // fewest authors covering >50% of changed lines
	Lines  int      `json:"lines"`
	Owners []string `json:"owners"` // the authors counted in Factor, biggest first
}

// BusFactors computes, per directory at the given depth, the smallest
// number of authors whose changes account for more than half of all
// changed lines. Directories with no line changes are omitted.
func BusFactors(commits []Commit, depth int) []BusFactor {
	res := resolver{}
	type key struct{ dir, who string }
	lines := map[key]int{}
	total := map[string]int{}
	for _, c := range commits {
		for _, fc := range c.Files {
			cur := res.resolve(fc.Path)
			res.note(fc)
			n := fc.Added + fc.Deleted
			d := dirAt(cur, depth)
			lines[key{d, c.Email}] += n
			total[d] += n
		}
	}
	perDir := map[string][]struct {
		who string
		n   int
	}{}
	for k, n := range lines {
		perDir[k.dir] = append(perDir[k.dir], struct {
			who string
			n   int
		}{k.who, n})
	}
	var out []BusFactor
	for dir, list := range perDir {
		if total[dir] == 0 {
			continue
		}
		sort.Slice(list, func(i, j int) bool {
			if list[i].n != list[j].n {
				return list[i].n > list[j].n
			}
			return list[i].who < list[j].who
		})
		bf := BusFactor{Dir: dir, Lines: total[dir]}
		covered := 0
		for _, e := range list {
			bf.Owners = append(bf.Owners, e.who)
			covered += e.n
			if covered*2 > total[dir] {
				break
			}
		}
		bf.Factor = len(bf.Owners)
		out = append(out, bf)
	}
	sort.Slice(out, func(i, j int) bool {
		if out[i].Factor != out[j].Factor {
			return out[i].Factor < out[j].Factor
		}
		if out[i].Lines != out[j].Lines {
			return out[i].Lines > out[j].Lines
		}
		return out[i].Dir < out[j].Dir
	})
	return out
}

// Activity counts commits by weekday (0=Sunday) and by "YYYY-MM" month.
type Activity struct {
	ByWeekday [7]int         `json:"by_weekday"`
	ByMonth   map[string]int `json:"by_month"`
}

func ActivityOf(commits []Commit) Activity {
	a := Activity{ByMonth: map[string]int{}}
	for _, c := range commits {
		a.ByWeekday[c.Time.Weekday()]++
		a.ByMonth[c.Time.Format("2006-01")]++
	}
	return a
}
