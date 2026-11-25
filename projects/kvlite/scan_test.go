package kvlite

import "testing"

func TestKeysAndScan(t *testing.T) {
	s := open(t, t.TempDir(), Options{})
	defer s.Close()
	for _, k := range []string{"user:2", "user:1", "order:1", "user:3"} {
		s.Put([]byte(k), []byte("v-"+k))
	}
	s.Delete([]byte("user:3"))

	keys, _ := s.Keys([]byte("user:"))
	if len(keys) != 2 || string(keys[0]) != "user:1" || string(keys[1]) != "user:2" {
		t.Fatalf("keys = %q", keys)
	}
	all, _ := s.Keys(nil)
	if len(all) != 3 {
		t.Fatalf("all = %q", all)
	}

	var seen []string
	s.Scan(nil, func(k, v []byte) bool {
		seen = append(seen, string(k)+"="+string(v))
		return len(seen) < 2
	})
	if len(seen) != 2 || seen[0] != "order:1=v-order:1" {
		t.Fatalf("seen = %q", seen)
	}
}
