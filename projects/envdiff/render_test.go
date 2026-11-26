package main

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func fixture() ([]Env, []Row) {
	envs := []Env{
		{"dev", map[string]string{"HOST": "localhost", "DB_PASSWORD": "pw1", "ONLYDEV": "1"}},
		{"prod", map[string]string{"HOST": "example.com", "DB_PASSWORD": "pw2"}},
	}
	return envs, Compare(envs)
}

func TestRenderTableMasksSecrets(t *testing.T) {
	envs, rows := fixture()
	var b bytes.Buffer
	RenderTable(&b, envs, rows, RenderOpts{})
	out := b.String()
	if strings.Contains(out, "pw1") || !strings.Contains(out, "****(") {
		t.Errorf("secret leaked or not masked:\n%s", out)
	}
	if !strings.Contains(out, "(missing)") || !strings.Contains(out, "example.com") {
		t.Errorf("unexpected output:\n%s", out)
	}
	b.Reset()
	RenderTable(&b, envs, rows, RenderOpts{ShowSecrets: true})
	if !strings.Contains(b.String(), "pw1") {
		t.Error("ShowSecrets should reveal")
	}
}

func TestRenderOnlyDiffAndMarkdown(t *testing.T) {
	envs := []Env{{"a", map[string]string{"X": "1", "Y": "p|q"}}, {"b", map[string]string{"X": "1", "Y": "z"}}}
	var b bytes.Buffer
	RenderMarkdown(&b, envs, Compare(envs), RenderOpts{OnlyDiff: true})
	out := b.String()
	if strings.Contains(out, "| X ") || !strings.Contains(out, `p\|q`) || !strings.Contains(out, "| --- |") {
		t.Errorf("markdown:\n%s", out)
	}
}

func TestRenderJSON(t *testing.T) {
	envs, rows := fixture()
	var b bytes.Buffer
	if err := RenderJSON(&b, envs, rows, nil, RenderOpts{}); err != nil {
		t.Fatal(err)
	}
	var got struct {
		Rows []struct {
			Key    string
			Values map[string]*string
		}
	}
	if err := json.Unmarshal(b.Bytes(), &got); err != nil {
		t.Fatal(err)
	}
	for _, r := range got.Rows {
		if r.Key == "ONLYDEV" && r.Values["prod"] != nil {
			t.Error("missing should be null")
		}
		if r.Key == "DB_PASSWORD" && strings.Contains(*r.Values["dev"], "pw1") {
			t.Error("leak")
		}
	}
}
