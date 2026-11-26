package main

import (
	"encoding/json"
	"fmt"
	"os"
	"time"
)

// Check describes one monitored target.
type Check struct {
	Name         string   `json:"name"`
	Type         string   `json:"type"` // "http" or "tcp"
	Target       string   `json:"target"`
	Interval     Duration `json:"interval"`
	Timeout      Duration `json:"timeout"`
	ExpectStatus int      `json:"expect_status,omitempty"`
	ExpectBody   string   `json:"expect_body,omitempty"`
}

// Config is the top-level config file.
type Config struct {
	Listen     string   `json:"listen"`
	DB         string   `json:"db"`
	Retention  Duration `json:"retention"`
	WebhookURL string   `json:"webhook_url,omitempty"`
	FlapAfter  int      `json:"flap_after"`
	Checks     []Check  `json:"checks"`
}

// Duration wraps time.Duration to parse strings like "30s" from JSON.
type Duration time.Duration

func (d *Duration) UnmarshalJSON(b []byte) error {
	var s string
	if err := json.Unmarshal(b, &s); err != nil {
		return fmt.Errorf("duration must be a string like \"30s\"")
	}
	v, err := time.ParseDuration(s)
	if err != nil {
		return err
	}
	*d = Duration(v)
	return nil
}

func (d Duration) MarshalJSON() ([]byte, error) {
	return json.Marshal(time.Duration(d).String())
}

func (d Duration) D() time.Duration { return time.Duration(d) }

// ParseConfig decodes, applies defaults and validates a config.
func ParseConfig(data []byte) (*Config, error) {
	var c Config
	dec := json.NewDecoder(bytesReader(data))
	dec.DisallowUnknownFields()
	if err := dec.Decode(&c); err != nil {
		return nil, fmt.Errorf("parse config: %w", err)
	}
	if c.Listen == "" {
		c.Listen = ":8080"
	}
	if c.DB == "" {
		c.DB = "pingboard.db"
	}
	if c.Retention == 0 {
		c.Retention = Duration(7 * 24 * time.Hour)
	}
	if c.FlapAfter <= 0 {
		c.FlapAfter = 2
	}
	seen := map[string]bool{}
	for i := range c.Checks {
		ch := &c.Checks[i]
		if ch.Name == "" {
			return nil, fmt.Errorf("check %d: name is required", i)
		}
		if seen[ch.Name] {
			return nil, fmt.Errorf("check %q: duplicate name", ch.Name)
		}
		seen[ch.Name] = true
		if ch.Type != "http" && ch.Type != "tcp" {
			return nil, fmt.Errorf("check %q: type must be http or tcp", ch.Name)
		}
		if ch.Target == "" {
			return nil, fmt.Errorf("check %q: target is required", ch.Name)
		}
		if ch.Interval == 0 {
			ch.Interval = Duration(30 * time.Second)
		}
		if ch.Interval.D() < time.Second {
			return nil, fmt.Errorf("check %q: interval must be >= 1s", ch.Name)
		}
		if ch.Timeout == 0 {
			ch.Timeout = Duration(10 * time.Second)
		}
	}
	return &c, nil
}

// LoadConfig reads and parses a config file.
func LoadConfig(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ParseConfig(data)
}
