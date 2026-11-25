package gitstats

import "testing"

func TestAuthors(t *testing.T) {
	cs := []Commit{
		commit("b@x", 1, FileChange{Path: "a", Added: 2, Deleted: 1}),
		commit("a@x", 2, FileChange{Path: "a", Added: 5}),
		commit("b@x", 5),
	}
	as := Authors(cs)
	if len(as) != 2 || as[0].Email != "b@x" || as[0].Commits != 2 || as[0].Added != 2 || as[0].Deleted != 1 {
		t.Fatalf("%+v", as)
	}
	if !as[0].First.Equal(t0.AddDate(0, 0, -5)) {
		t.Fatalf("first %v", as[0].First)
	}
}

func TestBusFactor(t *testing.T) {
	cs := []Commit{
		commit("a", 1, FileChange{Path: "core/x.go", Added: 80}, FileChange{Path: "ui/y.go", Added: 10}),
		commit("b", 2, FileChange{Path: "core/x.go", Added: 10}, FileChange{Path: "ui/y.go", Added: 10}),
		commit("c", 3, FileChange{Path: "core/z.go", Added: 10}, FileChange{Path: "ui/w.go", Added: 10}),
		commit("a", 4, FileChange{Path: "empty/bin", Binary: true}),
	}
	bf := BusFactors(cs, 1)
	if len(bf) != 2 {
		t.Fatalf("want 2 dirs (binary-only dir omitted), got %+v", bf)
	}
	if bf[0].Dir != "core" || bf[0].Factor != 1 || bf[0].Owners[0] != "a" {
		t.Fatalf("core: %+v", bf[0])
	}
	if bf[1].Dir != "ui" || bf[1].Factor != 2 { // a=10,b=10,c=10 of 30 -> two cover 20 > 15
		t.Fatalf("ui: %+v", bf[1])
	}
}

func TestActivity(t *testing.T) {
	// t0 is a Thursday.
	a := ActivityOf([]Commit{commit("a", 0), commit("a", 1), commit("a", 40)})
	if a.ByWeekday[4] != 1 || a.ByWeekday[3] != 1 || a.ByMonth["2026-01"] != 1 || a.ByMonth["2025-12"] != 1 || a.ByMonth["2025-11"] != 1 {
		t.Fatalf("%+v", a)
	}
}
