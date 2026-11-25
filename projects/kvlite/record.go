package kvlite

import (
	"encoding/binary"
	"errors"
	"hash/crc32"
	"io"
)

// On-disk record layout (little endian):
//
//	crc32   uint32  checksum of everything after this field
//	keyLen  uint32
//	valLen  uint32  tombstoneLen marks a deletion (no value bytes follow)
//	key     [keyLen]byte
//	value   [valLen]byte
const (
	headerSize   = 12
	tombstoneLen = ^uint32(0)

	// Limits keep a corrupt header from triggering a huge allocation.
	MaxKeySize   = 1 << 16
	MaxValueSize = 1 << 28
)

var (
	// ErrCorrupt means a record failed its checksum or had an impossible header.
	ErrCorrupt = errors.New("kvlite: corrupt record")
	crcTable   = crc32.MakeTable(crc32.Castagnoli)
)

type record struct {
	key       []byte
	value     []byte
	tombstone bool
}

// encodedSize is the number of bytes the record occupies on disk.
func (r record) encodedSize() int {
	n := headerSize + len(r.key)
	if !r.tombstone {
		n += len(r.value)
	}
	return n
}

func (r record) encode() []byte {
	buf := make([]byte, r.encodedSize())
	binary.LittleEndian.PutUint32(buf[4:], uint32(len(r.key)))
	if r.tombstone {
		binary.LittleEndian.PutUint32(buf[8:], tombstoneLen)
	} else {
		binary.LittleEndian.PutUint32(buf[8:], uint32(len(r.value)))
	}
	copy(buf[headerSize:], r.key)
	if !r.tombstone {
		copy(buf[headerSize+len(r.key):], r.value)
	}
	binary.LittleEndian.PutUint32(buf[0:], crc32.Checksum(buf[4:], crcTable))
	return buf
}

// readRecord reads one record from r. It returns io.EOF only when the stream
// ends exactly on a record boundary; a partial record yields ErrCorrupt so the
// caller can treat it as a torn write.
func readRecord(r io.Reader) (rec record, size int, err error) {
	var hdr [headerSize]byte
	n, err := io.ReadFull(r, hdr[:])
	if err == io.EOF && n == 0 {
		return record{}, 0, io.EOF
	}
	if err != nil {
		return record{}, 0, ErrCorrupt
	}
	keyLen := binary.LittleEndian.Uint32(hdr[4:])
	valLen := binary.LittleEndian.Uint32(hdr[8:])
	tomb := valLen == tombstoneLen
	if tomb {
		valLen = 0
	}
	if keyLen > MaxKeySize || valLen > MaxValueSize {
		return record{}, 0, ErrCorrupt
	}
	body := make([]byte, keyLen+valLen)
	if _, err := io.ReadFull(r, body); err != nil {
		return record{}, 0, ErrCorrupt
	}
	sum := crc32.Update(crc32.Update(0, crcTable, hdr[4:]), crcTable, body)
	if sum != binary.LittleEndian.Uint32(hdr[0:]) {
		return record{}, 0, ErrCorrupt
	}
	rec = record{key: body[:keyLen], value: body[keyLen:], tombstone: tomb}
	return rec, headerSize + len(body), nil
}
