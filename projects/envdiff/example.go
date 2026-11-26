package main

import "sort"

// CheckAgainst returns the template keys absent from env, and the env keys
// absent from the template. Both lists are sorted.
func CheckAgainst(tmpl, env Env) (missing, extra []string) {
	for k := range tmpl.Vars {
		if _, ok := env.Vars[k]; !ok {
			missing = append(missing, k)
		}
	}
	for k := range env.Vars {
		if _, ok := tmpl.Vars[k]; !ok {
			extra = append(extra, k)
		}
	}
	sort.Strings(missing)
	sort.Strings(extra)
	return
}
