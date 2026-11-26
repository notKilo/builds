package main

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

// Notifier posts transitions to a webhook as JSON.
type Notifier struct {
	URL    string
	Client *http.Client
}

type webhookPayload struct {
	Check string `json:"check"`
	State string `json:"state"`
	Error string `json:"error,omitempty"`
	Text  string `json:"text"`
	Time  string `json:"time"`
}

func (n *Notifier) Notify(ctx context.Context, tr Transition) error {
	if n == nil || n.URL == "" {
		return nil
	}
	state := "up"
	text := fmt.Sprintf("%s is back UP", tr.Check)
	if !tr.Up {
		state = "down"
		text = fmt.Sprintf("%s is DOWN: %s", tr.Check, tr.Error)
	}
	body, _ := json.Marshal(webhookPayload{Check: tr.Check, State: state, Error: tr.Error, Text: text, Time: time.Now().UTC().Format(time.RFC3339)})
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, n.URL, bytes.NewReader(body))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	c := n.Client
	if c == nil {
		c = &http.Client{Timeout: 10 * time.Second}
	}
	resp, err := c.Do(req)
	if err != nil {
		return err
	}
	resp.Body.Close()
	if resp.StatusCode >= 300 {
		return fmt.Errorf("webhook returned %d", resp.StatusCode)
	}
	return nil
}
