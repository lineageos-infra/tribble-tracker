package db

import (
	"database/sql"
	"fmt"
	"time"

	_ "github.com/lib/pq"
)

type postgresClient struct {
	db *sql.DB
}

func NewPostgresClient(DatabaseUrl string) (*postgresClient, error) {
	db, err := sql.Open("postgres", DatabaseUrl)
	if err != nil {
		return nil, err
	}

	client := &postgresClient{db: db}
	return client, nil

}

func (c *postgresClient) Close() {
	c.db.Close()
}

func (c *postgresClient) InsertStatistic(stat Statistic) error {
	stmt, err := c.db.Prepare(`
	INSERT INTO stats(device_id, model, version_raw, country, carrier, carrier_id) 
		VALUES ($1, $2, $3, $4, $5, $6)
		ON CONFLICT (device_id) DO UPDATE
		SET model=$2, version_raw=$3, country=$4, carrier = $5, carrier_id = $6, submit_time=now();
	`)

	if err != nil {
		return err
	}

	_, err = stmt.Exec(stat.Hash, stat.Name, stat.Version, stat.Country, stat.Carrier, stat.CarrierId)
	if err != nil {
		return err
	}

	return nil
}

func (c *postgresClient) GetMostPopular(field string, column string, filterable string) ([]Stat, error) {
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
	var rows *sql.Rows
	if column == "" {
		stmt, err := c.db.Prepare(fmt.Sprintf(`
			SELECT %s, count(%s) FROM stats
			GROUP BY %s
			ORDER BY count DESC
			LIMIT 250
			`, field, field, field))
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
			SELECT %s, count(%s) FROM stats
			WHERE %s = $1
			GROUP BY %s
			ORDER BY count DESC
			LIMIT 250
		`, field, field, column, field))
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
	return result, nil
}
func (c *postgresClient) GetCount(column string, filterable string) (int, error) {
	whitelist := map[string]interface{}{
		"version": nil,
		"model":   nil,
		"country": nil,
		"carrier": nil,
	}

	var row *sql.Row
	if column == "" {
		row = c.db.QueryRow(`SELECT count(device_id) FROM stats`)
	} else {
		if _, ok := whitelist[column]; !ok {
			// column wasn't valid, reject
			return 0, fmt.Errorf("invalid column: %s", column)
		}
		stmt, err := c.db.Prepare(fmt.Sprintf(`SELECT count(device_id) FROM stats WHERE %s = $1`, column))
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
func (c *postgresClient) DropOld() error {
	stmt, err := c.db.Prepare(`DELETE FROM stats WHERE submit < $1`)
	if err != nil {
		return err
	}
	_, err = stmt.Exec(stmt, time.Now().Add(-90*24*time.Hour))
	return err

}

func (c *postgresClient) GetBanned() (*Banned, error) {
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

func (b *Banned) Update(client *postgresClient) error {
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
