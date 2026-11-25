package kvlite

import (
	"bytes"
	"io"
	"testing"
)

func TestRecordRoundTrip(t *testing.T) {
	cases := []record{
		{key: []byte("a"), value: []byte("1")},
		{key: []byte("empty-value"), value: []byte{}},
		{key: []byte("gone"), tombstone: true},
		{key: []byte("big"), value: bytes.Repeat([]byte("x"), 100000)},
	}
	var buf bytes.Buffer
	for _, c := range cases {
		buf.Write(c.encode())
	}
	for _, want := range cases {
		got, size, err := readRecord(&buf)
		if err != nil {
			t.Fatalf("read %q: %v", want.key, err)
		}
		if size != want.encodedSize() {
			t.Errorf("size = %d, want %d", size, want.encodedSize())
		}
		if !bytes.Equal(got.key, want.key) || !bytes.Equal(got.value, want.value) || got.tombstone != want.tombstone {
			t.Errorf("got %+v, want %+v", got, want)
		}
	}
	if _, _, err := readRecord(&buf); err != io.EOF {
		t.Fatalf("expected EOF at boundary, got %v", err)
	}
}

func TestRecordDetectsBitFlip(t *testing.T) {
	enc := record{key: []byte("k"), value: []byte("value")}.encode()
	for i := range enc {
		bad := append([]byte(nil), enc...)
		bad[i] ^= 0x01
		if _, _, err := readRecord(bytes.NewReader(bad)); err == nil {
			t.Fatalf("flip at byte %d went undetected", i)
		}
	}
}

func TestRecordTruncated(t *testing.T) {
	enc := record{key: []byte("k"), value: []byte("value")}.encode()
	for n := 1; n < len(enc); n++ {
		if _, _, err := readRecord(bytes.NewReader(enc[:n])); err != ErrCorrupt {
			t.Fatalf("truncated to %d bytes: err = %v, want ErrCorrupt", n, err)
		}
	}
}
