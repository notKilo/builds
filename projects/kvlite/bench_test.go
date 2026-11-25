package kvlite

import (
	"fmt"
	"testing"
)

func benchStore(b *testing.B, opts Options) *Store {
	s, err := Open(b.TempDir(), opts)
	if err != nil {
		b.Fatal(err)
	}
	b.Cleanup(func() { s.Close() })
	return s
}

func BenchmarkPut(b *testing.B) {
	s := benchStore(b, Options{})
	val := make([]byte, 256)
	b.SetBytes(int64(len(val)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if err := s.Put([]byte(fmt.Sprintf("key-%d", i%10000)), val); err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkGet(b *testing.B) {
	s := benchStore(b, Options{})
	val := make([]byte, 256)
	for i := 0; i < 10000; i++ {
		s.Put([]byte(fmt.Sprintf("key-%d", i)), val)
	}
	b.SetBytes(int64(len(val)))
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if _, err := s.Get([]byte(fmt.Sprintf("key-%d", i%10000))); err != nil {
			b.Fatal(err)
		}
	}
}
