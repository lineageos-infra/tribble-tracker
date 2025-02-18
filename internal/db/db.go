package db

import (
	"database/sql"
	"fmt"
	"hash/fnv"
	"os"
	"strings"
	"time"

	_ "github.com/mattn/go-sqlite3"
	"github.com/TwiN/gocache/v2"
)

type sqlite3Client struct {
	cache *gocache.Cache
	db *sql.DB
}

func NewSqlite3Client(DatabasePath string) (*sqlite3Client, error) {
	db, err := sql.Open("sqlite3", fmt.Sprintf("file:%s?_journal_mode=WAL", DatabasePath))
	if err != nil {
		return nil, err
	}
	cache := gocache.NewCache().WithMaxSize(1000)
	cache.StartJanitor()
	client := &sqlite3Client{cache: cache, db: db}
	err = client.CreateIfNotExists()
	if err != nil {
		fmt.Printf("Failed to create tables/indexes: %s", err)
	}
	return client, nil
}

func (c *sqlite3Client) Close() {
	c.cache.StopJanitor()
	c.db.Close()
}

func (c *sqlite3Client) CreateIfNotExists() error {
	file, err := os.ReadFile("schema.sql")
	if err != nil {
		return err
	}
	statements := strings.Split(string(file), ";")
	for _, statement := range statements {
		if statement == "" {
			continue
		}
		fmt.Printf("Running statement %s\n", statement)
		_, err := c.db.Exec(statement)
		if err != nil {
			return err
		}
	}
	return nil
}

func (c *sqlite3Client) cacheKey(args ...string) string {
	hash := fnv.New64a()
	for _, arg := range args {
		hash.Write([]byte(arg))
	}
	return fmt.Sprint(hash.Sum64())
}

func (c *sqlite3Client) InsertStatistic(stat Statistic, version string, official bool) error {
	stmt, err := c.db.Prepare(`
	INSERT INTO stats(device_id, model, version_raw, country, carrier, carrier_id, version, official) 
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
		ON CONFLICT (device_id) DO UPDATE
		SET model=$2, version_raw=$3, country=$4, carrier = $5, carrier_id = $6, submit_time=current_timestamp, version=$7, official=$8;
	`)

	if err != nil {
		return err
	}

	_, err = stmt.Exec(stat.Hash, stat.Name, stat.Version, stat.Country, stat.Carrier, stat.CarrierId, version, official)
	if err != nil {
		return err
	}

	return nil
}

func (c *sqlite3Client) GetMostPopular(field string, column string, filterable string) ([]Stat, error) {
	whitelist := map[string]interface{}{
		"version": nil,
		"model":   nil,
		"country": nil,
		"carrier": nil,
	}
	if _, ok := whitelist[field]; !ok {
		// field wasn't valid, reject
		return nil, fmt.Errorf("invalid field: %s", field)
	}

	cacheKey := c.cacheKey(field, column, filterable)
	value, exists := c.cache.Get(cacheKey)
	if exists {
		return value.([]Stat), nil
	}

	var rows *sql.Rows
	if column == "" {
		stmt, err := c.db.Prepare(fmt.Sprintf(`
			SELECT %s, COUNT(*) as count FROM stats
			GROUP BY %s
			ORDER BY count DESC
			LIMIT 250
			`, field, field))
		if err != nil {
			return nil, err
		}

		rows, err = stmt.Query()
		if err != nil {
			return nil, err
		}
	} else {
		if _, ok := whitelist[column]; !ok {
			// column wasn't valid, reject
			return nil, fmt.Errorf("invalid column: %s", column)
		}

		stmt, err := c.db.Prepare(fmt.Sprintf(`
			SELECT %s, count(*) as count FROM stats
			WHERE %s = $1
			GROUP BY %s
			ORDER BY count DESC
			LIMIT 250
		`, field, column, field))
		if err != nil {
			return nil, err
		}
		rows, err = stmt.Query(filterable)
		if err != nil {
			return nil, err
		}
	}
	defer rows.Close()
	var result []Stat
	for rows.Next() {
		var name string
		var count int
		err := rows.Scan(&name, &count)
		if err != nil {
		}
		result = append(result, Stat{Item: name, Count: count})

	}
	err := rows.Err()
	if err != nil {
		return nil, err
	}
	c.cache.SetWithTTL(cacheKey, result, 6 * time.Hour)
	return result, nil
}
func (c *sqlite3Client) GetCount(column string, filterable string) (int, error) {
	whitelist := map[string]interface{}{
		"version": nil,
		"model":   nil,
		"country": nil,
		"carrier": nil,
	}

	var row *sql.Row
	if column == "" {
		row = c.db.QueryRow(`SELECT count(*) FROM stats`)
	} else {
		if _, ok := whitelist[column]; !ok {
			// column wasn't valid, reject
			return 0, fmt.Errorf("invalid column: %s", column)
		}
		stmt, err := c.db.Prepare(fmt.Sprintf(`SELECT count(*) FROM stats WHERE %s = $1`, column))
		if err != nil {
			return 0, err
		}
		row = stmt.QueryRow(filterable)
	}
	err := row.Err()
	if err != nil {
		return 0, err
	}

	var count int
	row.Scan(&count)
	return count, nil

}
func (c *sqlite3Client) DropOld() error {
	stmt, err := c.db.Prepare(`DELETE FROM stats WHERE submit_time < $1`)
	if err != nil {
		return err
	}
	_, err = stmt.Exec(time.Now().Add(-90 * 24 * time.Hour))
	return err

}

func (c *sqlite3Client) GetBanned() (*Banned, error) {
	rows, err := c.db.Query(`SELECT version, model FROM banned`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	banned := Banned{}
	banned.Models = make(map[string]bool)
	banned.Versions = make(map[string]bool)
	for rows.Next() {
		var version sql.NullString
		var model sql.NullString
		err := rows.Scan(&version, &model)
		if err != nil {
			return nil, err
		}
		if version.Valid {
			banned.Versions[version.String] = true
		}
		if model.Valid {
			banned.Models[model.String] = true
		}
	}
	err = rows.Err()
	if err != nil {
		return nil, err
	}
	return &banned, nil
}

func (b *Banned) Update(client *sqlite3Client) error {
	rows, err := client.db.Query(`SELECT version, model FROM banned`)
	if err != nil {
		return err
	}
	defer rows.Close()
	for rows.Next() {
		var version sql.NullString
		var model sql.NullString
		err := rows.Scan(&version, &model)
		if err != nil {
			return err
		}
		if version.Valid {
			b.Versions[version.String] = true
		}
		if model.Valid {
			b.Models[model.String] = true
		}
	}
	err = rows.Err()
	if err != nil {
		return err
	}
	return nil

}
