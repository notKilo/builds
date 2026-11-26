package main

import "strings"

var secretHints = []string{"secret", "password", "passwd", "token", "apikey", "api_key", "private", "credential", "auth", "dsn"}

// IsSecretKey guesses from the key name whether the value is sensitive.
func IsSecretKey(key string) bool {
	k := strings.ToLower(key)
	for _, h := range secretHints {
		if strings.Contains(k, h) {
			return true
		}
	}
	return strings.HasSuffix(k, "_key") || strings.HasSuffix(k, ".key")
}

// Mask hides a value but keeps its length class visible, so a changed
// secret can still be told from an unchanged one via fingerprint.
func Mask(v string) string {
	if v == "" {
		return ""
	}
	return "****(" + fingerprint(v) + ")"
}

// fingerprint is a short FNV-1a hash: equal secrets show equal tags
// without revealing the value.
func fingerprint(v string) string {
	h := uint32(2166136261)
	for i := 0; i < len(v); i++ {
		h ^= uint32(v[i])
		h *= 16777619
	}
	const hex = "0123456789abcdef"
	b := make([]byte, 4)
	for i := range b {
		b[i] = hex[(h>>(uint(i)*4))&0xf]
	}
	return string(b)
}

var placeholders = []string{"changeme", "change_me", "todo", "xxx", "your_", "<", "replace", "example", "fixme"}

// IsPlaceholder reports values that look like unfilled template text.
func IsPlaceholder(v string) bool {
	l := strings.ToLower(v)
	for _, p := range placeholders {
		if strings.Contains(l, p) {
			return true
		}
	}
	return false
}

// Warning is a per-key, per-environment hygiene finding.
type Warning struct {
	Key, Env, Reason string
}

// Lint finds empty and placeholder values in each environment.
func Lint(envs []Env, rows []Row) []Warning {
	var out []Warning
	for _, r := range rows {
		for i, e := range envs {
			if !r.Present[i] {
				continue
			}
			switch v := r.Values[i]; {
			case v == "":
				out = append(out, Warning{r.Key, e.Name, "empty value"})
			case IsPlaceholder(v):
				out = append(out, Warning{r.Key, e.Name, "placeholder value"})
			}
		}
	}
	return out
}
