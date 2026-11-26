package main

import "testing"

func TestIsSecretKey(t *testing.T) {
	for k, want := range map[string]bool{"DB_PASSWORD": true, "stripe.api_key": true, "SIGNING_KEY": true, "PORT": false, "HOST": false, "AUTH_URL": true} {
		if IsSecretKey(k) != want {
			t.Errorf("%s: want %v", k, want)
		}
	}
}

func TestMaskFingerprint(t *testing.T) {
	if Mask("a") != Mask("a") || Mask("a") == Mask("b") {
		t.Error("fingerprint should be stable and distinguish values")
	}
	if Mask("") != "" {
		t.Error("empty stays empty")
	}
	if got := Mask("hunter2"); len(got) != 10 || got[:5] != "****(" {
		t.Errorf("shape: %q", got)
	}
}

func TestLint(t *testing.T) {
	envs := []Env{{"dev", map[string]string{"A": "", "B": "CHANGEME", "C": "ok"}}, {"prod", map[string]string{"C": "ok"}}}
	w := Lint(envs, Compare(envs))
	if len(w) != 2 || w[0].Reason != "empty value" || w[1].Reason != "placeholder value" {
		t.Fatalf("%+v", w)
	}
}
