// Package main implements envdiff, a tool that compares configuration
// files across environments.
package main

import (
	"fmt"
	"strings"
)

// ParseDotenv parses .env style content (optionally with `export` prefixes)
// into an ordered key list and a value map.
func ParseDotenv(src string) (map[string]string, error) {
	out := map[string]string{}
	lines := strings.Split(strings.ReplaceAll(src, "\r\n", "\n"), "\n")
	for i := 0; i < len(lines); i++ {
		line := strings.TrimSpace(lines[i])
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		line = strings.TrimPrefix(line, "export ")
		eq := strings.IndexByte(line, '=')
		if eq <= 0 {
			return nil, fmt.Errorf("line %d: expected KEY=VALUE", i+1)
		}
		key := strings.TrimSpace(line[:eq])
		if !validKey(key) {
			return nil, fmt.Errorf("line %d: invalid key %q", i+1, key)
		}
		rest := strings.TrimLeft(line[eq+1:], " \t")
		val, consumed, err := parseValue(rest, lines, i)
		if err != nil {
			return nil, fmt.Errorf("line %d: %w", i+1, err)
		}
		i += consumed
		out[key] = val
	}
	return out, nil
}

func validKey(k string) bool {
	for i, r := range k {
		switch {
		case r == '_' || r == '.' || r == '-':
		case r >= 'A' && r <= 'Z', r >= 'a' && r <= 'z':
		case r >= '0' && r <= '9' && i > 0:
		default:
			return false
		}
	}
	return k != ""
}

// parseValue handles unquoted, single-quoted and double-quoted values.
// Double-quoted values may span lines; consumed is the number of extra
// lines read.
func parseValue(rest string, lines []string, idx int) (string, int, error) {
	if rest == "" {
		return "", 0, nil
	}
	switch rest[0] {
	case '\'':
		end := strings.IndexByte(rest[1:], '\'')
		if end < 0 {
			return "", 0, fmt.Errorf("unterminated single quote")
		}
		return rest[1 : 1+end], 0, nil
	case '"':
		var sb strings.Builder
		cur := rest[1:]
		consumed := 0
		for {
			for j := 0; j < len(cur); j++ {
				switch cur[j] {
				case '\\':
					if j+1 < len(cur) {
						j++
						sb.WriteString(unescape(cur[j]))
					}
				case '"':
					return sb.String(), consumed, nil
				default:
					sb.WriteByte(cur[j])
				}
			}
			if idx+consumed+1 >= len(lines) {
				return "", 0, fmt.Errorf("unterminated double quote")
			}
			consumed++
			sb.WriteByte('\n')
			cur = lines[idx+consumed]
		}
	}
	// Unquoted: strip trailing " # comment".
	if i := strings.Index(rest, " #"); i >= 0 {
		rest = rest[:i]
	}
	return strings.TrimSpace(rest), 0, nil
}

func unescape(c byte) string {
	switch c {
	case 'n':
		return "\n"
	case 't':
		return "\t"
	case 'r':
		return "\r"
	}
	return string(c)
}
