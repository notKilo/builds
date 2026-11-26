package main

import (
	"database/sql"
	"fmt"
	"sort"
	"time"

	_ "modernc.org/sqlite"
)

// Store persists check results in SQLite.
type Store struct{ db *sql.DB }

// OpenStore opens (and migrates) the database at path.
func OpenStore(path string) (*Store, error) {
	db, err := sql.Open("sqlite", path)
	if err != nil {
		return nil, err
	}
	db.SetMaxOpenConns(1) // SQLite: serialise writers, keeps :memory: coherent
	_, err = db.Exec(`CREATE TABLE IF NOT EXISTS results (
		id INTEGER PRIMARY KEY AUTOINCREMENT,
		check_name TEXT NOT NULL,
		ts INTEGER NOT NULL,
		up INTEGER NOT NULL,
		latency_us INTEGER NOT NULL,
		status INTEGER NOT NULL,
		error TEXT NOT NULL
	);
	CREATE INDEX IF NOT EXISTS results_check_ts ON results(check_name, ts);`)
	if err != nil {
		db.Close()
		return nil, fmt.Errorf("migrate: %w", err)
	}
	return &Store{db: db}, nil
}

func (s *Store) Close() error { return s.db.Close() }

// Insert records a result.
func (s *Store) Insert(r Result) error {
	up := 0
	if r.Up {
		up = 1
	}
	_, err := s.db.Exec(`INSERT INTO results(check_name, ts, up, latency_us, status, error) VALUES (?,?,?,?,?,?)`,
		r.Check, r.Time.UnixMilli(), up, r.Latency.Microseconds(), r.Status, r.Error)
	return err
}

// Recent returns up to limit results for a check, newest first.
func (s *Store) Recent(check string, limit int) ([]Result, error) {
	rows, err := s.db.Query(`SELECT ts, up, latency_us, status, error FROM results
		WHERE check_name = ? ORDER BY ts DESC, id DESC LIMIT ?`, check, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var out []Result
	for rows.Next() {
		var ts, up, lat int64
		r := Result{Check: check}
		if err := rows.Scan(&ts, &up, &lat, &r.Status, &r.Error); err != nil {
			return nil, err
		}
		r.Time = time.UnixMilli(ts)
		r.Up = up == 1
		r.Latency = time.Duration(lat) * time.Microsecond
		out = append(out, r)
	}
	return out, rows.Err()
}

// Stats summarises a check over a time window.
type Stats struct {
	Total     int
	Up        int
	UptimePct float64
	P50, P95  time.Duration
}

// Stats computes uptime and latency percentiles (of successful checks) since the given time.
func (s *Store) Stats(check string, since time.Time) (Stats, error) {
	rows, err := s.db.Query(`SELECT up, latency_us FROM results WHERE check_name = ? AND ts >= ?`,
		check, since.UnixMilli())
	if err != nil {
		return Stats{}, err
	}
	defer rows.Close()
	var st Stats
	var lats []int64
	for rows.Next() {
		var up, lat int64
		if err := rows.Scan(&up, &lat); err != nil {
			return Stats{}, err
		}
		st.Total++
		if up == 1 {
			st.Up++
			lats = append(lats, lat)
		}
	}
	if err := rows.Err(); err != nil {
		return Stats{}, err
	}
	if st.Total > 0 {
		st.UptimePct = 100 * float64(st.Up) / float64(st.Total)
	}
	sort.Slice(lats, func(i, j int) bool { return lats[i] < lats[j] })
	st.P50 = percentile(lats, 0.50)
	st.P95 = percentile(lats, 0.95)
	return st, nil
}

// percentile uses the nearest-rank method on sorted microsecond values.
func percentile(sorted []int64, p float64) time.Duration {
	if len(sorted) == 0 {
		return 0
	}
	rank := int(p*float64(len(sorted)) + 0.999999999)
	if rank < 1 {
		rank = 1
	}
	if rank > len(sorted) {
		rank = len(sorted)
	}
	return time.Duration(sorted[rank-1]) * time.Microsecond
}

// Prune deletes results older than the cutoff and returns how many were removed.
func (s *Store) Prune(before time.Time) (int64, error) {
	res, err := s.db.Exec(`DELETE FROM results WHERE ts < ?`, before.UnixMilli())
	if err != nil {
		return 0, err
	}
	return res.RowsAffected()
}
