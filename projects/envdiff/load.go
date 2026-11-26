package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// Env is one named environment's flattened key/value set.
type Env struct {
	Name string
	Vars map[string]string
}

// LoadFile reads a config file, choosing the parser from its extension
// (.json) or falling back to dotenv/export syntax. The environment name is
// the file's base name.
func LoadFile(path string) (Env, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Env{}, err
	}
	var vars map[string]string
	if strings.EqualFold(filepath.Ext(path), ".json") {
		vars, err = ParseJSON(data)
	} else {
		vars, err = ParseDotenv(string(data))
	}
	if err != nil {
		return Env{}, fmt.Errorf("%s: %w", path, err)
	}
	return Env{Name: filepath.Base(path), Vars: vars}, nil
}

// ParseJSON flattens a JSON object into dotted keys. Arrays become
// key.0, key.1, ...; null becomes the empty string.
func ParseJSON(data []byte) (map[string]string, error) {
	var root any
	if err := json.Unmarshal(data, &root); err != nil {
		return nil, err
	}
	obj, ok := root.(map[string]any)
	if !ok {
		return nil, fmt.Errorf("top-level JSON value must be an object")
	}
	out := map[string]string{}
	flatten("", obj, out)
	return out, nil
}

func flatten(prefix string, v any, out map[string]string) {
	join := func(k string) string {
		if prefix == "" {
			return k
		}
		return prefix + "." + k
	}
	switch t := v.(type) {
	case map[string]any:
		for k, c := range t {
			flatten(join(k), c, out)
		}
	case []any:
		for i, c := range t {
			flatten(join(fmt.Sprint(i)), c, out)
		}
	case nil:
		out[prefix] = ""
	case string:
		out[prefix] = t
	case float64:
		out[prefix] = strings.TrimRight(strings.TrimRight(fmt.Sprintf("%f", t), "0"), ".")
	default:
		out[prefix] = fmt.Sprint(t)
	}
}

// Status classifies one key across all environments.
type Status int

const (
	Same    Status = iota // present everywhere with identical values
	Differs               // present everywhere, values differ
	Missing               // absent from at least one environment
)

func (s Status) String() string { return [...]string{"same", "differs", "missing"}[s] }

// Row is one key's values across environments, in env order. Present[i]
// tells whether env i defines the key at all.
type Row struct {
	Key     string
	Values  []string
	Present []bool
	Status  Status
}

// Compare builds the key-by-environment matrix, sorted by key.
func Compare(envs []Env) []Row {
	keys := map[string]bool{}
	for _, e := range envs {
		for k := range e.Vars {
			keys[k] = true
		}
	}
	sorted := make([]string, 0, len(keys))
	for k := range keys {
		sorted = append(sorted, k)
	}
	sort.Strings(sorted)

	rows := make([]Row, 0, len(sorted))
	for _, k := range sorted {
		r := Row{Key: k, Values: make([]string, len(envs)), Present: make([]bool, len(envs))}
		for i, e := range envs {
			r.Values[i], r.Present[i] = e.Vars[k]
		}
		r.Status = classify(r)
		rows = append(rows, r)
	}
	return rows
}

func classify(r Row) Status {
	for _, p := range r.Present {
		if !p {
			return Missing
		}
	}
	for _, v := range r.Values[1:] {
		if v != r.Values[0] {
			return Differs
		}
	}
	return Same
}
